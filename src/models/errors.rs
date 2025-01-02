// models/errors.rs

#[derive(Debug)]
pub enum SystemError {
    MemoryInfoUnavailable,
    DiskInfoUnavailable,
    NetworkTrafficUnavailable,
    TemperatureSensorsUnavailable,
    DockerConnectionFailed,
    DockerListContainersFailed,
    UptimeUnavailable,
}

impl SystemError {
    pub fn message(&self) -> &str {
        match self {
            Self::MemoryInfoUnavailable => "Failed to retrieve memory information.",
            Self::DiskInfoUnavailable => "Failed to retrieve disk information.",
            Self::NetworkTrafficUnavailable => "Failed to retrieve network traffic information.",
            Self::TemperatureSensorsUnavailable => "Failed to read temperature sensors.",
            //Self::SSHStatusCheckFailed => "Failed to check SSH service status.",
            Self::DockerConnectionFailed => "Failed to connect to Docker.",
            Self::DockerListContainersFailed => "Failed to list Docker containers.",
            Self::UptimeUnavailable => "Failed to retrieve uptime information.",
        }
    }
}