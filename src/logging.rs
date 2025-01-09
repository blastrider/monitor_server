use chrono::Local;
use fern::Dispatch;
use log::{info, LevelFilter};
use crate::config::Config;

pub fn init_logging(config: &Config) -> Result<(), fern::InitError> {
    let log_level = match config.log_level.to_lowercase().as_str() {
        "debug" => LevelFilter::Debug,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        _ => LevelFilter::Info, // Valeur par défaut si invalide
    };

    Dispatch::new()
        .level(log_level)
        .chain(std::io::stdout())
        .chain(fern::log_file(&config.log_file)?)
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

    info!("Logging initialized successfully with level: {}", config.log_level);
    Ok(())
}