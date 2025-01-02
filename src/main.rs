mod config;
mod handlers;
mod models;
mod services;

use actix_web::{middleware::{Logger, NormalizePath, TrailingSlash}, web, App, HttpServer};
use config::init_logging;
use handlers::status::get_status;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging().expect("Failed to initialize logging");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .route("/status", web::get().to(get_status))
    })
    .bind("127.0.0.1:8550")?
    .run()
    .await
}