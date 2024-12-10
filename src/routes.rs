use actix_web::{web, HttpResponse, Responder};
use crate::models::{DepositRequest, WithdrawRequest};
use crate::services::{deposit_to_contract, withdraw_from_contract, check_contract_balance};
use crate::utils::create_response;
use ethers::types::U256;

// Root endpoint returning a simple "Rust service is running" message
pub async fn root() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"message": "Rust service is running"}))
}

// Handler for depositing ETH into the contract
#[utoipa::path(
    post,
    path = "/deposit",
    request_body = DepositRequest,
    responses(
        (status = 200, description = "Deposit confirmed"),
        (status = 500, description = "Server error")
    )
)]
pub async fn deposit(req: web::Json<DepositRequest>) -> impl Responder {
    let amount: U256 = req.amount.parse().unwrap();
    let result = deposit_to_contract(amount).await;
    create_response(result)
}

// Handler for withdrawing ETH from the contract
#[utoipa::path(
    post,
    path = "/withdraw",
    request_body = WithdrawRequest,
    responses(
        (status = 200, description = "Withdraw confirmed"),
        (status = 500, description = "Server error")
    )
)]
pub async fn withdraw(req: web::Json<WithdrawRequest>) -> impl Responder {
    let amount: U256 = req.amount.parse().unwrap();
    let result = withdraw_from_contract(amount).await;
    create_response(result)
}

// Handler to check the balance of the user's contract
#[utoipa::path(
    post,
    path = "/check_balance",
    responses(
        (status = 200, description = "Returns wallet address's balance"),
        (status = 500, description = "Server error")
    )
)]
pub async fn check_balance() -> impl Responder {
    let result = check_contract_balance().await;
    create_response(result)
}
