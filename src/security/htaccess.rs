use std::{collections::HashMap, fs};

pub fn load_htpasswd(file_path: &str) -> HashMap<String, String> {
    let content = fs::read_to_string(file_path).expect("Unable to read htpasswd file");
    content
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}
