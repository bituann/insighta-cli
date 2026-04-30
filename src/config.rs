use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: String,
}

fn credentials_path() -> Result<PathBuf> {
    let home = home_dir().context("Could not find home directory")?;
    Ok(home.join(".insighta").join("credentials.json"))
}

pub fn load() -> Result<Credentials> {
    let path = credentials_path()?;
    let content = fs::read_to_string(&path)
        .context("Not logged in. Run `insighta login` first.")?;
    serde_json::from_str(&content).context("Corrupted credentials. Run `insighta login`.")
}

pub fn save(credentials: &Credentials) -> Result<()> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string_pretty(credentials)?)?;
    Ok(())
}

pub fn clear() -> Result<()> {
    let path = credentials_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}