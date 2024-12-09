use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use utoipa::{ToSchema, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
use ethers::prelude::*;
use dotenv::dotenv;
use std::env;

abigen!(
    StakeContract,
    "./abi/StakeContract.abi", // Path to the ABI file for interacting with the contract
    event_derives(serde::Deserialize, serde::Serialize)
);

// Request and response structures for deposit, withdraw, and balance check
#[derive(Deserialize)]
struct DepositRequest {
    amount: String, // Amount to deposit, provided as a string
}

#[derive(Deserialize)]
struct WithdrawRequest {
    amount: String, // Amount to withdraw, provided as a string
}

#[derive(Serialize)]
struct JsonResponse {
    status: String,             // Status of the operation (success/error)
    transaction_hash: Option<String>, // Optional transaction hash if available
    balance: Option<U256>,        // Optional balance if returning a balance
    message: Option<String>,      // Optional message in case of errors
}

// Common helper function to initialize the provider, wallet, and client
// This reduces duplication of this setup logic across multiple functions
fn initialize_client() -> Result<SignerMiddleware<Provider<Http>, LocalWallet>, Box<dyn std::error::Error>> {
    // Initialize the provider using the Infura URL from the environment
    let provider = Provider::<Http>::try_from(env::var("INFURA_URL")?)?;

    // Initialize the wallet using the private key from the environment
    let wallet: LocalWallet = env::var("PRIVATE_KEY")?
        .parse::<LocalWallet>()? // Parse private key
        .with_chain_id(Chain::Holesky); // Set the blockchain's chain ID (Holesky testnet)

    // Create a SignerMiddleware to combine the provider and wallet for signed transactions
    let client = SignerMiddleware::new(provider, wallet);

    Ok(client)
}

// Common function to generate responses for both success and error cases
// This handles wrapping the result into a consistent JsonResponse format
fn create_response<T>(
    result: Result<Option<T>, Box<dyn std::error::Error>>, // Result that may return an Option
    success_status: &str,       // The status to return on success (e.g., "success")
    error_status: &str,         // The status to return on error (e.g., "error")
    success_message: Option<String>, // Optional message for success
    error_message: Option<String>,   // Optional error message if the transaction failed
    transaction_hash: Option<String>, // Optional transaction hash if available
) -> HttpResponse
where
    T: Serialize, // T must be serializable to be used in the response
{
    match result {
        Ok(Some(tx)) => HttpResponse::Ok().json(JsonResponse {
            status: success_status.to_string(),
            transaction_hash,    // Include the transaction hash in the response
            balance: None,        // No balance in this case
            message: success_message, // Include success message if available
        }),
        Ok(None) => HttpResponse::InternalServerError().json(JsonResponse {
            status: error_status.to_string(),
            transaction_hash: None,  // No transaction hash available in case of failure
            balance: None,
            message: error_message,  // Include error message
        }),
        Err(e) => HttpResponse::InternalServerError().json(JsonResponse {
            status: error_status.to_string(),
            transaction_hash: None,
            balance: None,
            message: Some(e.to_string()), // Include the error message as a string
        }),
    }
}

// Root endpoint returning a simple "Rust service is running" message
async fn root() -> impl Responder {
    HttpResponse::Ok().json(json!({"message": "Rust service is running"}))
}

// Define OpenAPI structure with documentation for the endpoints
#[derive(OpenApi)]
#[openapi(
    paths(deposit, withdraw, check_balance),  // Register the deposit, withdraw, and balance check paths
    components(schemas(DepositRequest, WithdrawRequest)), // Register the request schemas for deposit/withdraw
    tags(
        (name = "Staking API", description = "Endpoints for user staking") // Tag for API categorization
    )
)]
struct ApiDoc;

// Main function to set up and run the Actix web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load environment variables from .env file

    HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")  // Swagger UI for API documentation
                    .url("/api-docs/openapi.json", ApiDoc::openapi())  // OpenAPI spec
            )
            .route("/", web::get().to(root))  // Root route
            .route("/deposit", web::post().to(deposit))  // Deposit route
            .route("/withdraw", web::post().to(withdraw))  // Withdraw route
            .route("/check_balance", web::post().to(check_balance))  // Check balance route
    })
        .bind("127.0.0.1:8081")?  // Binding server to local address
        .run()
        .await
}

// Handler for depositing ETH
#[utoipa::path(
    post,
    path = "/deposit",
    request_body = DepositRequest,
    responses(
        (status = 200, description = "Deposit confirmed"),
        (status = 500, description = "Server error")
    )
)]
async fn deposit(req: web::Json<DepositRequest>) -> impl Responder {
    let amount: U256 = req.amount.parse().unwrap();  // Parse the amount from the request
    let result = deposit_to_contract(amount).await; // Call the deposit function
    create_response(result, "success", "error", None, Some("Transaction failed to return a receipt.".to_string()), None) // Handle the response
}

// Handler for withdrawing ETH
#[utoipa::path(
    post,
    path = "/withdraw",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdraw confirmed"),
        (status = 500, description = "Server error")
    )
)]
async fn withdraw(req: web::Json<WithdrawRequest>) -> impl Responder {
    let amount: U256 = req.amount.parse().unwrap();  // Parse the amount from the request
    let result = withdraw_from_contract(amount).await; // Call the withdraw function
    create_response(result, "success", "error", None, Some("Transaction failed to return a receipt.".to_string()), None) // Handle the response
}

// Handler for checking balance of a specific user
#[utoipa::path(
    post,
    path = "/check_balance",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdraw confirmed"),
        (status = 500, description = "Server error")
    )
)]
async fn check_balance() -> impl Responder {
    let result = check_contract_balance().await; // Call the function to check the balance
    create_response(result, "success", "error", None, None, None) // Handle the response
}

// Deposit to contract function: Sends a deposit transaction to the contract
async fn deposit_to_contract(amount: U256) -> Result<Option<TransactionReceipt>, Box<dyn std::error::Error>> {
    let client = initialize_client()?;  // Initialize the client (provider + wallet)
    let contract_addr: Address = env::var("CONTRACT_ADDRESS")?.parse()?;  // Get the contract address from env variable
    let contract = StakeContract::new(contract_addr, Arc::new(client.clone()));  // Initialize the contract instance
    let tx = contract.deposit().value(amount).send().await?.await?;  // Send deposit transaction and wait for receipt
    println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);  // Log the transaction receipt
    Ok(tx)  // Return the transaction receipt
}

// Withdraw from contract function: Sends a withdraw transaction to the contract
async fn withdraw_from_contract(amount: U256) -> Result<Option<TransactionReceipt>, Box<dyn std::error::Error>> {
    let client = initialize_client()?;  // Initialize the client (provider + wallet)
    let contract_addr: Address = env::var("CONTRACT_ADDRESS")?.parse()?;  // Get the contract address from env variable
    let contract = StakeContract::new(contract_addr, Arc::new(client.clone()));  // Initialize the contract instance
    let tx = contract.withdraw(amount).send().await?.await?;  // Send withdraw transaction and wait for receipt
    println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);  // Log the transaction receipt
    Ok(tx)  // Return the transaction receipt
}

// Check contract balance function: Calls the contract to check the balance
async fn check_contract_balance() -> Result<Option<U256>, Box<dyn std::error::Error>> {
    let client = initialize_client()?;  // Initialize the client (provider + wallet)
    let contract_addr: Address = env::var("CONTRACT_ADDRESS")?.parse()?;  // Get the contract address from env variable
    let contract = StakeContract::new(contract_addr, Arc::new(client.clone()));  // Initialize the contract instance
    let value = contract.check_balance().call().await?;  // Call the contract method to check balance
    println!("Balance is {}", value);  // Log the balance
    Ok(Some(value))  // Return the balance wrapped in Some
}
