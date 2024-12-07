use utoipa::OpenApi;

// OpenAPI documentation setup for the service
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::deposit,
        crate::routes::deposit_with_wallet,
        crate::routes::withdraw,
        crate::routes::withdraw_with_wallet,
        crate::routes::check_balance,
        crate::routes::check_balance_with_wallet
    ),
    // Request body schemas
    components(schemas(
        crate::models::DepositOrWithdraw,
        crate::models::DepositOrWithdrawWithWallet,
        crate::models::CheckWithWallet
    )),
    tags(
        (name = "Staking API", description = "Endpoints for user staking") // Group endpoints under 'Staking API' tag
    )
)]
pub struct ApiDoc;
