mod config;
mod handlers;
mod models;
mod security;
mod services;

use actix_web::{middleware::Logger, web, App, HttpServer};
use config::init_logging;
use handlers::status::{get_service_status, get_status};
use security::{auth::AuthMiddleware, htaccess::load_htpasswd};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let htpasswd = Arc::new(load_htpasswd("/etc/monitor_server/htpasswd"));
    init_logging().expect("Failed to initialize logging");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(AuthMiddleware::new(Arc::clone(&htpasswd))) // Pas de duplication réelle
            .route("/status", web::get().to(get_status)) // Statut système
            .route(
                "/status/{service}",
                web::get().to(get_service_status), // Statut d'un service spécifique
            )
    })
    .bind("127.0.0.1:8550")?
    .run()
    .await
}
