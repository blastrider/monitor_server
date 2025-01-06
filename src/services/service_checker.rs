use log::warn;
use serde::Deserialize;
use std::fs;
use std::process::Command;

#[derive(Deserialize)]
struct Config {
    services: Vec<String>,
}

pub fn is_service_active(service: &str) -> bool {
    match Command::new("systemctl")
        .arg("is-active")
        .arg(service)
        .output()
    {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim() == "active",
        Err(_) => {
            warn!("Failed to check status of service {}", service);
            false
        }
    }
}

pub fn check_services(config_path: &str) -> Vec<String> {
    let services = load_services_from_config(config_path);
    services
        .into_iter()
        .filter(|service| is_service_active(service))
        .collect()
}

pub fn load_services_from_config(path: &str) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => {
            warn!("Failed to read configuration file at {}", path);
            return vec![];
        }
    };

    let config: Config = match toml::from_str(&content) {
        Ok(config) => config,
        Err(_) => {
            warn!("Failed to parse TOML configuration file.");
            return vec![];
        }
    };

    config.services
}
