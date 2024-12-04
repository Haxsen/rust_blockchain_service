use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Request and response structures for deposit, withdraw, and balance check
#[derive(Deserialize, Serialize, ToSchema)]
pub struct DepositOrWithdraw {
    pub amount: String, // Amount to deposit (string representation)
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DepositOrWithdrawWithWallet {
    pub amount: String, // Amount to deposit (string representation)
    pub privatekey: String, // Private key for client execution
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CheckWithWallet {
    pub privatekey: String, // Private key for client execution
}

#[derive(Serialize)]
pub struct JsonResponse {
    pub status: String,             // Status of the operation (success/error)
    pub message: Option<String>,    // Optional message for error or result info
}
