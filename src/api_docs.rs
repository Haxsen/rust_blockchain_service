use utoipa::OpenApi;

// OpenAPI documentation setup for the service
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::deposit,
        crate::routes::withdraw,
        crate::routes::check_balance
    ),
    // Request body schemas
    components(schemas(
        crate::models::DepositRequest,
        crate::models::WithdrawRequest
    )),
    tags(
        (name = "Staking API", description = "Endpoints for user staking") // Group endpoints under 'Staking API' tag
    )
)]
pub struct ApiDoc;
