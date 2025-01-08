# WDL Discord Bot

A Discord bot written in Rust using the Serenity framework.

## Features

- Quote management
- Message scraping
- MySQL database integration
- Logging system

## Prerequisites

- Rust toolchain
- MySQL database
- cross-rs (for cross-compilation)
- Discord bot token

## Dependencies

Key dependencies include:
- serenity: Discord API framework
- sqlx: Async MySQL database operations
- tokio: Async runtime
- clap: Command line argument parsing
- chrono: Date and time functionality

## Building

### Local Build

```bash
cargo build --release
```

### Cross Compilation

To build for ARM64 Linux (e.g., Raspberry Pi):

1. Install cross-rs:
```bash
cargo install cross --git https://github.com/cross-rs/cross
```

2. Build using cross:
```bash
cross build --target x86_64-unknown-linux-gnu
```

## Database Setup

1. Create a MySQL database
2. Run the migrations from the `migrations/` directory:
   - 20240107140717_0001_discord_messages.sql
   - 20240107181232_add_name_column.sql
   - 20240110173402_change_utf_format.sql

## Environment Variables

Create a `.env` file with:
```
DISCORD_TOKEN=your_bot_token
DATABASE_URL=mysql://user:password@localhost/database_name
```

## Running

```bash
cargo run --release
```
