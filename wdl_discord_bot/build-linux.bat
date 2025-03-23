@echo off
setlocal enabledelayedexpansion

echo Building Rust project for Linux using WSL...

:: Check if WSL is installed
wsl --list >nul 2>&1
if errorlevel 1 (
    echo WSL is not installed or not running. Please install WSL first.
    echo See: https://docs.microsoft.com/en-us/windows/wsl/install
    exit /b 1
)

:: Ensure the necessary packages are installed in WSL
echo Checking for required packages in WSL...
wsl bash -c "if ! dpkg -l | grep -q build-essential; then sudo apt update && sudo apt install -y build-essential gcc-x86-64-linux-gnu binutils-x86-64-linux-gnu; fi"

:: Check if Rust is installed in WSL
echo Checking for Rust in WSL...
wsl bash -c "source $HOME/.cargo/env && rustc -V"  > nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Rust is not installed in WSL. Please install Rust in WSL first.
    echo Run the following commands in WSL:
    echo curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs ^| sh
    echo source $HOME/.cargo/env
    exit /b 1
)
echo Found Rust: 
wsl bash -c "rustc -V"

:: Set environment variables for cross-compilation
set CARGO_TARGET_DIR=./target/linux
set TARGET=x86_64-unknown-linux-gnu

:: Ensure .cargo directory exists
if not exist .cargo mkdir .cargo

:: Check if the Linux target is installed
echo Checking for Linux target in WSL...
wsl bash -c "rustup target list | grep -q 'x86_64-unknown-linux-gnu installed' || rustup target add x86_64-unknown-linux-gnu"

:: Build the project using WSL
echo Cross-compiling for Linux...
wsl cargo build --release --target=x86_64-unknown-linux-gnu

if %ERRORLEVEL% EQU 0 (
    echo Build successful!
    echo Linux binary located at: ./target/x86_64-unknown-linux-gnu/release/wdl_discord_bot
    
    :: Optionally copy the binary to a specific location
    if not exist bin mkdir bin
    wsl cp ./target/x86_64-unknown-linux-gnu/release/wdl_discord_bot ./bin/
    echo Binary copied to ./bin/wdl_discord_bot
) else (
    echo Build failed with error code: %ERRORLEVEL%
)

endlocal 