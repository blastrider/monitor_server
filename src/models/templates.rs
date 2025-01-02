use askama::Template;
use serde::Serialize;

#[derive(Template)]
#[template(path = "status.html")]
pub struct StatusTemplate {
    pub hostname: String,
    pub system_version: String,
    pub kernel_info: String,
    pub uptime: String,
    pub memory_used: String,
    pub memory_total: String,
    pub disk_available: String,
    pub disk_total: String,
    pub temperature: String,
    pub network_in: String,
    pub network_out: String,
    pub containers: Vec<ContainerStatus>,
    pub ssh_active: bool,
    pub current_year: u32,
    pub local_ip: String,
    pub public_ip: String,
}

#[derive(Serialize)]
pub struct ContainerStatus {
    pub image: String,
    pub state: String,
}