use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Request and response structures for deposit, withdraw, and balance check
#[derive(Deserialize, Serialize, ToSchema)]
pub struct DepositRequest {
    pub amount: String, // Amount to deposit (string representation)
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct WithdrawRequest {
    pub amount: String, // Amount to withdraw (string representation)
}

#[derive(Serialize)]
pub struct JsonResponse {
    pub status: String,             // Status of the operation (success/error)
    pub message: Option<String>,    // Optional message for error or result info
}
