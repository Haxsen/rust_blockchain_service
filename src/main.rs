use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex};
use utoipa::{ToSchema, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
use ethers::prelude::*;
use ethers::contract::Contract;
use ethers::abi::Abi;
use dotenv::dotenv;
use std::env;

// Define the request models and their schemas
#[derive(Deserialize, Serialize, ToSchema)]
struct RestakeRequest {
    user_id: i32,
    amount: f64,
}

#[derive(Deserialize, Serialize, ToSchema)]
struct ConfirmRequest {
    operation_id: i32,
}

// Define the response and the application state
struct AppState {
    operations: Arc<Mutex<Vec<RestakeOperation>>>,
}

#[derive(Clone, Serialize, ToSchema)]
struct RestakeOperation {
    operation_id: i32,
    user_id: i32,
    amount: f64,
    status: String,
}

async fn initiate_restake_on_chain(user_id: i32, amount: f64) -> Result<TxHash, Box<dyn std::error::Error>> {
    dotenv().ok();

    // Load environment variables
    let private_key = env::var("PRIVATE_KEY")?;
    let rpc_url = env::var("INFURA_URL")?;
    let contract_address: Address = env::var("CONTRACT_ADDRESS")?.parse()?;

    // Create wallet and provider
    let wallet: LocalWallet = private_key.parse()?;
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // Signer middleware with chain ID 420 (Holesky testnet)
    let client = SignerMiddleware::new(provider, wallet.with_chain_id(420u64));
    let client = Arc::new(client);

    // Parse the ABI from the file
    let abi_bytes = include_bytes!("../abi/RestakeContract.json");  // Raw byte array of the ABI

    // Use serde_json to parse the ABI JSON into the Abi type
    let abi: Abi = serde_json::from_slice(abi_bytes)?;

    // Create the contract instance
    let contract = Contract::new(contract_address, abi, client);

    // Call the `initiateRestake` method and store the method call result in a variable
    let method_call = contract.method::<_, U256>("initiateRestake", (user_id, (amount * 1e18) as u64))?;

    // Send the transaction and wait for it to be mined
    let tx = method_call.send().await?;

    // Wait for the transaction receipt and return the transaction hash
    let receipt = tx.await?;
    Ok(receipt.unwrap().transaction_hash)
}

// Define handlers and associate them with OpenAPI annotations
#[utoipa::path(
    post,
    path = "/start-restake",
    request_body = RestakeRequest,
    responses(
        (status = 200, description = "Restake operation initiated", body = RestakeOperation),
        (status = 500, description = "Server error")
    )
)]
async fn start_restake(req: web::Json<RestakeRequest>, data: web::Data<AppState>) -> impl Responder {
    let mut operations = data.operations.lock().unwrap();

    let new_operation = RestakeOperation {
        operation_id: operations.len() as i32 + 1,
        user_id: req.user_id,
        amount: req.amount,
        status: "pending".to_string(),
    };

    operations.push(new_operation.clone());

    // Blockchain interaction
    match initiate_restake_on_chain(req.user_id, req.amount).await {
        Ok(tx_hash) => {
            return HttpResponse::Ok().json(serde_json::json!({
            "message": "Restake initiated on blockchain",
            "operation_id": operations.len(),
            "transaction_hash": format!("{:?}", tx_hash)
        }));
        }
        Err(e) => {
            eprintln!("Blockchain interaction failed: {:?}", e);
            return HttpResponse::InternalServerError().body("Blockchain interaction failed");
        }
    }
}

#[utoipa::path(
    post,
    path = "/confirm-restake",
    request_body = ConfirmRequest,
    responses(
        (status = 200, description = "Restake operation confirmed"),
        (status = 404, description = "Operation not found"),
        (status = 500, description = "Server error")
    )
)]
async fn confirm_restake(req: web::Json<ConfirmRequest>, data: web::Data<AppState>) -> impl Responder {
    let mut operations = data.operations.lock().unwrap();

    if let Some(operation) = operations.iter_mut().find(|op| op.operation_id == req.operation_id) {
        operation.status = "completed".to_string();
        HttpResponse::Ok().json(json!({"status": "completed", "message": "Restake operation confirmed and completed"}))
    } else {
        HttpResponse::NotFound().json(json!({"status": "error", "message": "Operation not found"}))
    }
}

async fn root() -> impl Responder {
    HttpResponse::Ok().json(json!({"message": "Rust service is running"}))
}

// Define the OpenAPI structure and expose the endpoints
#[derive(OpenApi)]
#[openapi(
    paths(start_restake, confirm_restake),
    components(schemas(RestakeRequest, ConfirmRequest, RestakeOperation)),
    tags(
        (name = "Restake API", description = "Endpoints for restake operations")
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let app_state = web::Data::new(AppState {
        operations: Arc::new(Mutex::new(Vec::new())),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
            .route("/", web::get().to(root))
            .route("/start-restake", web::post().to(start_restake))
            .route("/confirm-restake", web::post().to(confirm_restake))
    })
        .bind("127.0.0.1:8081")?
        .run()
        .await
}
