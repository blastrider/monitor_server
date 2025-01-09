use serde::Deserialize;
use config::{Config as ConfigLoader, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default = "default_log_file")]
    pub log_file: String,

    #[serde(default = "default_server_address")]
    pub server_address: String,

    #[serde(default = "default_server_port")]
    pub server_port: u16,

    #[serde(default = "default_htpasswd_path")]
    pub htpasswd_path: String,

    #[serde(default = "default_services_path")]
    pub services_path: String,
}

impl Config {
    /// Charge la configuration depuis un fichier TOML, avec valeurs par défaut.
    pub fn from_file(file: &str) -> Result<Self, ConfigError> {
        let settings = ConfigLoader::builder()
            .add_source(File::with_name(file).required(false))
            .build()?;
        settings.try_deserialize::<Self>()
    }
}

/// Valeurs par défaut
fn default_server_address() -> String {
    "0.0.0.0".to_string()
}

fn default_server_port() -> u16 {
    8550
}

fn default_htpasswd_path() -> String {
    "/etc/monitor_server/htpasswd".to_string()
}

fn default_services_path() -> String {
    "/etc/monitor_server/services.toml".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_file() -> String {
    "server.log".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_defaults() {
        let config = Config::from_file("nonexistent_config").unwrap();
        assert_eq!(config.server_address, "0.0.0.0");
        assert_eq!(config.server_port, 8550);
        assert_eq!(config.htpasswd_path, "/etc/monitor_server/htpasswd");
        assert_eq!(config.services_path, "/etc/monitor_server/services.toml");
    }

    #[test]
    fn test_load_config_override() {
        let config = Config::from_file("test_config").unwrap();
        assert_eq!(config.server_address, "127.0.0.1");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.htpasswd_path, "/custom/path/htpasswd");
    }
}
