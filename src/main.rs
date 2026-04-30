mod api;
mod cli;
mod commands;
mod config;
mod display;
mod oauth;

use clap::Parser;
use colored::Colorize;

#[tokio::main]
async fn main() {
    let cli = cli::Cli::parse();
    if let Err(e) = cli::run(cli).await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}