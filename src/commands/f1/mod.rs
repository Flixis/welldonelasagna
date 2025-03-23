mod models;
mod api;
mod embed;
mod commands;
mod registry;

pub use commands::{handle_commands, check_upcoming_race};
pub use registry::register; 