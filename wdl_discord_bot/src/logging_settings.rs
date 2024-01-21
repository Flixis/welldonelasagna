use chrono::Local;
use tracing::Level;
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{fmt, layer::SubscriberExt, prelude::*, EnvFilter};
use whoami::{hostname, username};

pub fn setup_logging(log_folder: &str, app_name: &str, log_level: Level) {
    // Determine the log file name based on current date, time, PC name, PC user, and app name
    let pc_name = hostname();
    let pc_user = username();
    let date_time = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file_name = format!("{}_{}_{}_{}.log", date_time, pc_name, pc_user, app_name);

    // Set up file appender for logs
    let file_appender = rolling::daily(log_folder, log_file_name);
    let (non_blocking, _guard) = non_blocking(file_appender);

    // Set the subscriber with the desired log level and layers for console and file
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(fmt::layer().pretty()) // For terminal output
        .with(fmt::layer().with_writer(non_blocking).pretty()) // For file output
        .init();
}
