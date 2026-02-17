//! Warp FOSS - A free terminal with AI integration

mod ai;
mod config;
mod plugin;
mod terminal;
mod ui;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    println!("Warp FOSS v0.1.0");
    println!("🚧 Early development - not functional yet");
    
    Ok(())
}
