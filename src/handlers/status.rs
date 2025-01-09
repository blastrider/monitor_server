use crate::{
    models::{errors::SystemError, templates::StatusTemplate},
    services::{
        docker::get_containers,
        hardware::{
            get_disk_info, get_kernel_version, get_memory_info, get_network_traffic,
            get_system_version, get_temperature, get_uptime,
        },
        service_checker::{check_services, is_service_active, load_services_from_config},
    },
};
use actix_web::{body::BoxBody, web, HttpResponse, Responder};
use askama::Template;
use chrono::{Datelike, Local};
use get_if_addrs::get_if_addrs;
use log::{debug, error, info};
use reqwest::Client;
use crate::config::Config;

pub async fn get_service_status(path: web::Path<String>) -> impl Responder<Body = BoxBody> {
    
    let service = path.into_inner();
    if is_service_active(&service) {
        HttpResponse::Ok().body(format!("Service '{}' is active", service))
    } else {
        HttpResponse::InternalServerError().body(format!("Service '{}' is not active", service))
    }
}

pub async fn get_status(req: actix_web::HttpRequest) -> impl Responder<Body = BoxBody> {
    let config = Config::from_file("config").expect("Failed to load configuration");
    info!("Starting to gather system status");
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "Unknown".to_string());
    info!("Retrieved hostname: {}", hostname);

    let forwarded_for = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown");
    info!("Client IP (X-Forwarded-For): {}", forwarded_for);

    let kernel_version = get_kernel_version();
    info!("Kernel version: {}", kernel_version);

    let system_version = get_system_version();
    info!("System version: {}", system_version);

    let uptime = get_uptime().unwrap_or_else(|_| "Unknown".to_string());
    info!("Uptime: {}", uptime);

    let memory_info = get_memory_info().unwrap_or((0, 0));
    debug!(
        "Memory info: used: {}, total: {}",
        memory_info.0, memory_info.1
    );

    let disk_info = get_disk_info("/").unwrap_or((0, 0));
    debug!(
        "Disk info: available: {}, total: {}",
        disk_info.0, disk_info.1
    );

    let network_traffic = get_network_traffic().unwrap_or((0, 0));
    debug!(
        "Network traffic: received: {}, sent: {}",
        network_traffic.0, network_traffic.1
    );

    let temperature = get_temperature().unwrap_or_else(|_| "N/A".to_string());
    debug!("Temperature: {}", temperature);

    let containers = get_containers().await;
    info!("Docker containers retrieved: {}", containers.len());

    // Vérification des services
    let all_services = load_services_from_config(&config.services_path);
    let active_services = check_services(&config.services_path);

    let services_status = all_services
        .iter()
        .map(|service| (service.clone(), active_services.contains(service)))
        .collect();

    let inactive_services: Vec<String> = all_services
        .clone()
        .into_iter()
        .filter(|service| !active_services.contains(service))
        .collect();

    info!("{:?} services are active", active_services);
    if !inactive_services.is_empty() {
        info!("{:?} services are inactive", inactive_services);
    }

    let ip_addresses = get_ip_addresses()
        .await
        .unwrap_or(("Unknown".to_string(), "Unknown".to_string()));
    info!(
        "Local IP: {}, Public IP: {}",
        ip_addresses.0, ip_addresses.1
    );
    debug!(
        "Local IP: {}, Public IP: {}",
        ip_addresses.0, ip_addresses.1
    );

    let current_year = Local::now().year() as u32;

    let template = StatusTemplate {
        hostname,
        system_version,
        kernel_info: kernel_version,
        uptime,
        memory_used: format_size(memory_info.0),
        memory_total: format_size(memory_info.1),
        disk_available: format_size(disk_info.0),
        disk_total: format_size(disk_info.1),
        temperature,
        network_in: format_size(network_traffic.0),
        network_out: format_size(network_traffic.1),
        containers,
        current_year,
        local_ip: ip_addresses.0,
        public_ip: ip_addresses.1,
        services_status,
    };

    match template.render() {
        Ok(html) => {
            info!("Status page rendered successfully");
            HttpResponse::Ok().content_type("text/html").body(html)
        }
        Err(e) => {
            error!("Failed to render template: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

async fn get_ip_addresses() -> Result<(String, String), SystemError> {
    // Récupérer l'IP privée
    let private_ip = match get_if_addrs() {
        Ok(interfaces) => interfaces
            .into_iter()
            .find_map(|iface| {
                if !iface.is_loopback() {
                    match iface.addr {
                        get_if_addrs::IfAddr::V4(addr) => Some(addr.ip.to_string()),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Unknown".to_string()),
        Err(_) => "Unknown".to_string(),
    };

    // Récupérer l'IP publique (asynchrone)
    let client = Client::new();
    let public_ip = match client.get("https://api.ipify.org").send().await {
        Ok(response) => match response.text().await {
            Ok(ip) => ip,
            Err(_) => "Unknown".to_string(),
        },
        Err(_) => "Unknown".to_string(),
    };

    Ok((private_ip, public_ip))
}

// Fonction pour convertir une taille en unité lisible
fn format_size(bytes: u64) -> String {
    match bytes {
        b if b >= 1 << 30 => format!("{:.2} GB", b as f64 / (1 << 30) as f64),
        b if b >= 1 << 20 => format!("{:.2} MB", b as f64 / (1 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.2} KB", b as f64 / (1 << 10) as f64),
        _ => format!("{} B", bytes),
    }
}
