use anyhow::{bail, Context, Result};
use colored::Colorize;
use reqwest::Client;
use serde_json::Value;

use crate::{
    config::{self, Credentials},
    display,
    oauth::{generate_pkce, start_callback_server, CALLBACK_URL},
};

fn base_url() -> String {
    std::env::var("INSIGHTA_API_URL")
        .unwrap_or_else(|_| "https://intelligence-query-engine-production-f522.up.railway.app".to_string())
}

pub async fn login() -> Result<()> {
    let pkce = generate_pkce();
    let rx = start_callback_server()?;
    let redirect_uri = CALLBACK_URL;

    let pb = display::spinner("Fetching GitHub login URL...");

    let client = Client::new();
    let url = format!(
        "{}/auth/github?redirect_uri={}&code_challenge={}&code_challenge_method=S256&state={}",
        base_url(),
        urlencoding::encode(&redirect_uri),
        pkce.challenge,
		pkce.state
    );
    let github_url = client
        .get(&url)
        .send()
        .await
        .context("Could not reach backend")?
        .text()
        .await?;

    pb.finish_and_clear();

    display::info("Opening GitHub in your browser...");
    open::that(&github_url).context("Could not open browser. Visit this URL manually:")?;
    println!("  {}", github_url.dimmed());
    println!();

    let pb = display::spinner("Waiting for GitHub authentication...");

    let (code, returned_state) = tokio::task::spawn_blocking(move || {
		rx.recv_timeout(std::time::Duration::from_secs(300))
	})
	.await?
	.context("Login timed out. Please try again.")?;

	if returned_state != pkce.state {
		bail!("State mismatch — possible CSRF attack. Please try logging in again.");
	}

    pb.finish_and_clear();
    let pb = display::spinner("Completing login...");

    // Call backend callback with the GitHub code and PKCE verifier
    let callback_url = format!(
        "{}/auth/github/callback?code={}&code_verifier={}&redirect_uri={}",
        base_url(),
        code,
        pkce.verifier,
        urlencoding::encode(&redirect_uri)
    );

    let res: Value = client
        .get(&callback_url)
        .send()
        .await
        .context("Network error")?
        .json()
        .await
        .context("Invalid response from server")?;

    pb.finish_and_clear();

    let access_token = res["access_token"]
        .as_str()
        .context("No access token received")?;
    let refresh_token = res["refresh_token"]
        .as_str()
        .context("No refresh token received")?;

    config::save(&Credentials {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.to_string(),
    })?;

    display::success("Logged in successfully!");
    Ok(())
}

pub async fn logout() -> Result<()> {
    let creds = config::load()?;

    let pb = display::spinner("Logging out...");

    let client = Client::new();
    let _ = client
        .post(format!("{}/auth/logout", base_url()))
        .bearer_auth(&creds.access_token)
        .header("X-API-Version", "1")
        .send()
        .await;

    pb.finish_and_clear();

    config::clear()?;
    display::success("Logged out successfully.");
    Ok(())
}

pub async fn whoami() -> Result<()> {
    let mut client = crate::api::ApiClient::new()?;

    let pb = display::spinner("Fetching user info...");
    let data: Value = client.get("/auth/me").await?;
    pb.finish_and_clear();

    let user = &data["data"];
    println!();
    println!("  {}  {}", "Username:".dimmed(), user["username"].as_str().unwrap_or("—").bold());
    println!("  {}     {}", "Email:".dimmed(), user["email"].as_str().unwrap_or("—"));
    println!("  {}      {}", "Role:".dimmed(), format_role(user["role"].as_str().unwrap_or("—")));
    println!();
    Ok(())
}

fn format_role(role: &str) -> colored::ColoredString {
    match role {
        "admin" => role.purple().bold(),
        "analyst" => role.cyan().normal(),
        _ => role.normal(),
    }
}