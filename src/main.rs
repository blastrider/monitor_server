use get_if_addrs::get_if_addrs;
use std::{ffi::CString, fs, process::Command};

use actix_web::middleware::{Logger, NormalizePath, TrailingSlash};
use actix_web::{web, HttpResponse, HttpServer, Responder};
use askama::Template;
use bollard::{container::ListContainersOptions, Docker};
use chrono::{Datelike, Local};
use fern::Dispatch;
use log::{debug, error, info, warn};
use reqwest::Client;
use serde::Serialize;

// Définition des erreurs possibles
#[derive(Debug)]
enum SystemError {
    MemoryInfoUnavailable,
    DiskInfoUnavailable,
    NetworkTrafficUnavailable,
    TemperatureSensorsUnavailable,
    SSHStatusCheckFailed,
    DockerConnectionFailed,
    DockerListContainersFailed,
    UptimeUnavailable,
}

impl SystemError {
    fn message(&self) -> &str {
        match self {
            Self::MemoryInfoUnavailable => "Failed to retrieve memory information.",
            Self::DiskInfoUnavailable => "Failed to retrieve disk information.",
            Self::NetworkTrafficUnavailable => "Failed to retrieve network traffic information.",
            Self::TemperatureSensorsUnavailable => "Failed to read temperature sensors.",
            Self::SSHStatusCheckFailed => "Failed to check SSH service status.",
            Self::DockerConnectionFailed => "Failed to connect to Docker.",
            Self::DockerListContainersFailed => "Failed to list Docker containers.",
            Self::UptimeUnavailable => "Failed to retrieve uptime information.",
        }
    }
}

// Structure pour les conteneurs Docker
#[derive(Serialize)]
struct ContainerStatus {
    image: String,
    state: String,
}

// Template HTML pour la page de statut
#[derive(Template)]
#[template(path = "status.html")]

struct StatusTemplate {
    hostname: String,
    system_version: String,
    uptime: String,
    memory_used: String,
    memory_total: String,
    disk_available: String,
    disk_total: String,
    temperature: String,
    network_in: String,
    network_out: String,
    containers: Vec<ContainerStatus>,
    ssh_active: bool,
    current_year: u32,
    local_ip: String,
    public_ip: String,
    kernel_info: String,
}

fn get_kernel_version() -> String {
    std::process::Command::new("uname")
        .arg("-r")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|version| version.trim().to_string()) // Supprimer les espaces inutiles
        .unwrap_or_else(|| "Unknown Kernel".to_string())
}

// Fonction pour obtenir les adresses IP (privée et publique)
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

// Récupérer la version du système
fn get_system_version() -> String {
    fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("PRETTY_NAME="))
                .and_then(|line| line.split('=').nth(1))
                .map(|value| value.trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Unknown System".to_string())
}

// Fonction pour récupérer le temps de fonctionnement
fn get_uptime() -> Result<String, SystemError> {
    fs::read_to_string("/proc/uptime")
        .map_err(|_| {
            error!("{}", SystemError::UptimeUnavailable.message());
            SystemError::UptimeUnavailable
        })
        .and_then(|content| {
            let mut parts = content.split_whitespace();
            if let Some(uptime_seconds) = parts.next().and_then(|s| s.parse::<f64>().ok()) {
                let days = (uptime_seconds / 86400.0).floor() as u64;
                let hours = ((uptime_seconds % 86400.0) / 3600.0).floor() as u64;
                let minutes = ((uptime_seconds % 3600.0) / 60.0).floor() as u64;
                Ok(format!(
                    "{} days, {} hours, {} minutes",
                    days, hours, minutes
                ))
            } else {
                warn!("Uptime format invalid in /proc/uptime");
                Err(SystemError::UptimeUnavailable)
            }
        })
}

// Fonction pour initialiser les journaux
fn init_logging() -> Result<(), fern::InitError> {
    Dispatch::new()
        .level(log::LevelFilter::Debug)
        .level_for("actix_web", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("server.log")?)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .apply()?;
    info!("Logging initialized successfully");
    Ok(())
}

async fn get_status(req: actix_web::HttpRequest) -> impl Responder {
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

    let ssh_active = is_ssh_active();
    if ssh_active {
        info!("SSH service is active");
    } else {
        warn!("SSH service is inactive");
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
        ssh_active,
        current_year,
        local_ip: ip_addresses.0,
        public_ip: ip_addresses.1,
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

// Fonction pour convertir une taille en unité lisible
fn format_size(bytes: u64) -> String {
    match bytes {
        b if b >= 1 << 30 => format!("{:.2} GB", b as f64 / (1 << 30) as f64),
        b if b >= 1 << 20 => format!("{:.2} MB", b as f64 / (1 << 20) as f64),
        b if b >= 1 << 10 => format!("{:.2} KB", b as f64 / (1 << 10) as f64),
        _ => format!("{} B", bytes),
    }
}

// Fonction pour obtenir la mémoire totale et utilisée
fn get_memory_info() -> Result<(u64, u64), SystemError> {
    let meminfo = fs::read_to_string("/proc/meminfo").map_err(|_| {
        error!(
            "{}
",
            SystemError::MemoryInfoUnavailable.message()
        );
        SystemError::MemoryInfoUnavailable
    })?;
    let total_memory = extract_memory_value(&meminfo, "MemTotal")?;
    let free_memory = extract_memory_value(&meminfo, "MemAvailable")?;
    Ok((total_memory - free_memory, total_memory))
}

fn extract_memory_value(meminfo: &str, key: &str) -> Result<u64, SystemError> {
    meminfo
        .lines()
        .find(|line| line.starts_with(key))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u64>().ok())
        .map(|kb| kb * 1024)
        .ok_or_else(|| {
            warn!("Key {} not found in /proc/meminfo", key);
            SystemError::MemoryInfoUnavailable
        })
}

// Fonction pour obtenir l'espace disque
fn get_disk_info(path: &str) -> Result<(u64, u64), SystemError> {
    let c_path = CString::new(path).map_err(|_| {
        error!(
            "{}
",
            SystemError::DiskInfoUnavailable.message()
        );
        SystemError::DiskInfoUnavailable
    })?;
    let mut statvfs: libc::statvfs = unsafe { std::mem::zeroed() };

    (unsafe { libc::statvfs(c_path.as_ptr(), &mut statvfs) } == 0)
        .then(|| {
            (
                statvfs.f_bavail as u64 * statvfs.f_frsize as u64,
                statvfs.f_blocks as u64 * statvfs.f_frsize as u64,
            )
        })
        .ok_or_else(|| {
            error!(
                "{}
",
                SystemError::DiskInfoUnavailable.message()
            );
            SystemError::DiskInfoUnavailable
        })
}

// Fonction pour obtenir la température moyenne
fn get_temperature() -> Result<String, SystemError> {
    let entries = fs::read_dir("/sys/class/thermal/").map_err(|_| {
        warn!("Temperature sensors directory not found. This may be a VM environment.");
        SystemError::TemperatureSensorsUnavailable
    });

    // Si le répertoire n'est pas disponible, retourne une valeur par défaut
    let entries = match entries {
        Ok(e) => e,
        Err(_) => return Ok("Unavailable (VM environment)".to_string()),
    };

    let temperatures: Vec<f64> = entries
        .flatten()
        .filter_map(|entry| {
            fs::read_to_string(entry.path().join("temp"))
                .ok()
                .and_then(|s| s.trim().parse::<i64>().ok())
                .map(|temp| temp as f64 / 1000.0)
        })
        .collect();

    if temperatures.is_empty() {
        warn!("No temperature data found in /sys/class/thermal/. Returning default value.");
        Ok("Unavailable (VM environment)".to_string())
    } else {
        let avg_temp = temperatures.iter().sum::<f64>() / temperatures.len() as f64;
        debug!("Average temperature calculated: {:.2} °C", avg_temp);
        Ok(format!("{:.2} °C", avg_temp))
    }
}


// Fonction pour obtenir le trafic réseau
fn get_network_traffic() -> Result<(u64, u64), SystemError> {
    fs::read_to_string("/proc/net/dev")
        .map_err(|_| {
            error!(
                "{}
",
                SystemError::NetworkTrafficUnavailable.message()
            );
            SystemError::NetworkTrafficUnavailable
        })?
        .lines()
        .skip(2)
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            Ok((
                parts
                    .get(1)
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0),
                parts
                    .get(9)
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0),
            ))
        })
        .try_fold((0, 0), |(rx_total, tx_total), result| {
            result.map(|(rx, tx)| (rx_total + rx, tx_total + tx))
        })
}

// Fonction pour vérifier si le service SSH ou SSHD est actif
fn is_ssh_active() -> bool {
    let services = ["ssh", "sshd"];
    for service in &services {
        match Command::new("systemctl")
            .arg("is-active")
            .arg(service)
            .output()
        {
            Ok(output) => {
                if String::from_utf8_lossy(&output.stdout).trim() == "active" {
                    info!("Service {} is active", service);
                    return true;
                }
            }
            Err(_) => {
                warn!(
                    "{}",
                    format!("Failed to check status of service {}", service)
                );
            }
        }
    }

    error!(
        "{}
",
        SystemError::SSHStatusCheckFailed.message()
    );
    false
}

// Fonction pour obtenir la liste des conteneurs Docker
async fn get_containers() -> Vec<ContainerStatus> {
    match Docker::connect_with_local_defaults() {
        Ok(docker) => match docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
        {
            Ok(containers) => containers
                .into_iter()
                .map(|c| ContainerStatus {
                    image: c.image.unwrap_or_default(),
                    state: c.state.unwrap_or_default(),
                })
                .collect(),
            Err(_) => {
                error!(
                    "{}
",
                    SystemError::DockerListContainersFailed.message()
                );
                vec![]
            }
        },
        Err(_) => {
            error!(
                "{}
",
                SystemError::DockerConnectionFailed.message()
            );
            vec![]
        }
    }
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging().expect("Failed to initialize logging");

    info!("Starting the Actix Web server on 127.0.0.1:8550");
    HttpServer::new(|| {
        actix_web::App::new()
            .wrap(Logger::default()) // Ajoute des logs pour les requêtes
            .wrap(NormalizePath::new(TrailingSlash::Trim)) // Normalise les chemins
            .route("/status", web::get().to(get_status)) // Route principale
    })
    .bind("127.0.0.1:8550")? // Actix écoute sur 127.0.0.1:8550
    .run()
    .await
}
