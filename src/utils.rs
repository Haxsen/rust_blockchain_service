use ethers::prelude::*;
use std::env;
use std::error::Error;
use crate::models::JsonResponse;
use actix_web::HttpResponse;

// Helper function to initialize the Ethereum provider and wallet
pub fn initialize_client(mut privatekey: String) -> Result<SignerMiddleware<Provider<Http>, LocalWallet>, Box<dyn Error>> {
    let provider = Provider::<Http>::try_from(env::var("INFURA_URL")?)?;
    privatekey = if privatekey.is_empty() { env::var("PRIVATE_KEY").unwrap().to_string() } else { privatekey.to_string() };
    let wallet: LocalWallet = privatekey.parse::<LocalWallet>()?.with_chain_id(Chain::Holesky);
    let client = SignerMiddleware::new(provider, wallet);
    Ok(client)
}

// Common response generator for success and error scenarios
pub fn create_response(result: Result<Option<String>, Box<dyn std::error::Error>>) -> HttpResponse {
    match result {
        Ok(Some(tx)) => HttpResponse::Ok().json(JsonResponse {
            status: "success".to_string(),
            message: Some(tx),
        }),
        Ok(None) => HttpResponse::InternalServerError().json(JsonResponse {
            status: "error".to_string(),
            message: Some("Method returned nothing".to_string()),
        }),
        Err(e) => HttpResponse::InternalServerError().json(JsonResponse {
            status: "exception".to_string(),
            message: Some(e.to_string()),
        }),
    }
}

pub fn get_stake_contract_addr() -> Address {
    let contract_addr: Address = env::var("CONTRACT_ADDRESS")
        .expect("couldnt fetch CONTRACT_ADDRESS").parse().unwrap();
    contract_addr
}
