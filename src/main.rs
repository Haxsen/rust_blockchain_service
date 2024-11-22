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
    // TODO: Add real implementation
    Ok(TxHash::from_low_u64_be(123))
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

    // TODO: Add Blockchain interaction

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
