use log::{debug, info, warn};
use std::{collections::HashMap, fs}; // Importation pour les logs

pub fn load_htpasswd(file_path: &str) -> HashMap<String, String> {
    debug!("Loading htpasswd file from: {}", file_path);

    let content = match fs::read_to_string(file_path) {
        Ok(data) => {
            info!("Successfully read htpasswd file");
            data
        }
        Err(e) => {
            warn!("Failed to read htpasswd file: {}", e);
            panic!("Unable to read htpasswd file: {}", e);
        }
    };

    let entries: HashMap<String, String> = content
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                debug!("Parsed entry for user: {}", parts[0]);
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                warn!("Invalid line format in htpasswd file: {}", line);
                None
            }
        })
        .collect();

    debug!("Loaded {} entries from htpasswd file", entries.len());
    entries
}
