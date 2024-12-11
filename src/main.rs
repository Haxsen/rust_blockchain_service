use actix_web::{web, App, HttpServer};
use utoipa_swagger_ui::SwaggerUi;
use dotenv::dotenv;
use utoipa::OpenApi;
use crate::routes::{deposit, withdraw, check_balance, root};
use crate::api_docs::ApiDoc;

mod routes;
mod services;
mod models;
mod utils;
mod api_docs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load environment variables from .env file

    HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")  // Swagger UI for API documentation
                    .url("/api-docs/openapi.json", ApiDoc::openapi())  // OpenAPI spec for API docs
            )
            .route("/", web::get().to(root))  // Root route returning a simple message
            .route("/deposit", web::post().to(deposit))  // Deposit route
            .route("/withdraw", web::post().to(withdraw))  // Withdraw route
            .route("/check_balance", web::post().to(check_balance))  // Check balance route
    })
        .bind("127.0.0.1:8081")?  // Bind to localhost on port 8081
        .run()
        .await
}
