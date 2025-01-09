mod config;
mod logging;
mod handlers;
mod models;
mod security;
mod services;

use actix_web::{
    middleware::{Logger, NormalizePath, TrailingSlash},
    web, App, HttpServer,
};
use logging::init_logging;
use handlers::status::{get_service_status, get_status};
use security::{auth::AuthMiddleware, htaccess::load_htpasswd};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = config::Config::from_file("config").expect("Failed to load configuration");

    let htpasswd = Arc::new(load_htpasswd(&config.htpasswd_path));
    init_logging(&config).expect("Failed to initialize logging");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(AuthMiddleware::new(Arc::clone(&htpasswd)))
            .route("/status", web::get().to(get_status))
            .route("/status/{service}", web::get().to(get_service_status))
    })
    .bind(format!("{}:{}", config.server_address, config.server_port))?
    .run()
    .await
}
