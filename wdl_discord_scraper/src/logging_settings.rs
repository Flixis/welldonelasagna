use chrono::prelude::*;
use simplelog::*;
use std::fs;
use std::fs::File;
use std::path::Path;
use whoami;

/// App logging is setup with the following configuration:
/// 
/// Terminal logger -> Filter:Warn, Config:Default, TerminalMode: Mixed, ColorChoice: Auto
/// 
/// Write Logger -> Filter:Info, Config:Default, File: Create(filename)
/// 
/// filename -> {name}/{day-month-year_hour_minute}_{username}_{hostname}_{find_testlog}.log
pub fn setup_loggers(name: String) {
    let name = format!("{name}_logs");
    fs::create_dir_all(&name).expect("unable to create logging directory");
    let utc = Utc::now().format("%d-%m-%Y_%H_%M");
    let filename_string_creation = format!(
        "{}/{}_{}_{}_{}",
        name,
        utc,
        whoami::username(),
        whoami::hostname(),
        ".log"
    );
    let filename = Path::new(&filename_string_creation);

    //initialize loggers with settings
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(filename).expect("failed to create log file"),
        ),
    ])
    .expect("Couldn't initialize loggers");
}
