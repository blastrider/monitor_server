use chrono::Local;
use fern::Dispatch;
use log::info;

pub fn init_logging() -> Result<(), fern::InitError> {
    Dispatch::new()
        .level(log::LevelFilter::Info)
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