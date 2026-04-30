use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "insighta",
    about = "Insighta Labs CLI — query demographic profiles",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with GitHub
    Login,
    /// Log out of your account
    Logout,
    /// Show the currently logged-in user
    Whoami,
    /// Manage profiles
    Profiles {
        #[command(subcommand)]
        subcommand: ProfileCommands,
    },
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// List profiles with optional filters
    List {
        #[arg(long)]
        gender: Option<String>,
        #[arg(long)]
        country: Option<String>,
        #[arg(long = "age-group")]
        age_group: Option<String>,
        #[arg(long = "min-age")]
        min_age: Option<u32>,
        #[arg(long = "max-age")]
        max_age: Option<u32>,
        #[arg(long = "sort-by")]
        sort_by: Option<String>,
        #[arg(long, default_value = "asc")]
        order: String,
        #[arg(long, default_value = "1")]
        page: u32,
        #[arg(long, default_value = "10")]
        limit: u32,
    },
    /// Get a single profile by ID
    Get { id: String },
    /// Search profiles using natural language
    Search { query: String },
    /// Create a new profile (admin only)
    Create {
        #[arg(long)]
        name: String,
    },
    /// Delete a profile (admin only)
    Delete { id: String },
    /// Export profiles to a file
    Export {
        #[arg(long, default_value = "csv")]
        format: String,
        #[arg(long)]
        gender: Option<String>,
        #[arg(long)]
        country: Option<String>,
        #[arg(long = "age-group")]
        age_group: Option<String>,
    },
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Login => crate::commands::auth::login().await,
        Commands::Logout => crate::commands::auth::logout().await,
        Commands::Whoami => crate::commands::auth::whoami().await,
        Commands::Profiles { subcommand } => {
            crate::commands::profiles::run(subcommand).await
        }
    }
}