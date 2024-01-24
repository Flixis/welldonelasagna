use chrono::prelude::*;
use simplelog::*;
use std::fs::{self, OpenOptions};
use std::path::Path;
use whoami;

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME"); //<-- read from cargo.toml

/// App logging is setup with the following configuration:
///
/// Terminal logger -> Filter:Info, Config:Default, TerminalMode: Mixed, ColorChoice: Auto
///
/// Write Logger -> Filter:Info, Config:Defaulcart, File: Create(filename)
///
/// filename -> find_testlog_logs/{day-month-year_hour_minute}_{username}_{hostname}_{find_testlog}.log
pub fn setup_loggers() {
    let directory_name = format!("{}_logs", PACKAGE_NAME);
    fs::create_dir_all(&directory_name).expect("unable to create logging directory");
    let utc = Utc::now().format("%d-%m-%Y_%H_%M");
    let filename_string_creation = format!(
        "{}/{}_{}_{}_{}",
        directory_name,
        utc,
        whoami::username(),
        whoami::hostname(),
        ".log"
    );
    let filename = Path::new(&filename_string_creation);

    let file = OpenOptions::new()
        .create(true) // This will create the file if it does not exist
        .write(true) // Open the file in write mode
        .append(true) // Set the file to append mode
        .open(filename)
        .expect("failed to open or create log file");

    let config = ConfigBuilder::new()
        .add_filter_allow_str(PACKAGE_NAME)
        .build();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, config, file),
    ])
    .expect("Couldn't initialize loggers");
}
