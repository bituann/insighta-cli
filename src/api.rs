use anyhow::{bail, Context, Result};
use reqwest::{Client, Method};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::{self, Credentials};

fn base_url() -> String {
    std::env::var("INSIGHTA_API_URL")
        .unwrap_or_else(|_| "https://intelligence-query-engine-production-f522.up.railway.app".to_string())
}

pub struct ApiClient {
    client: Client,
    creds: Credentials,
}

impl ApiClient {
    pub fn new() -> Result<Self> {
        let creds = config::load()?;
        Ok(Self {
            client: Client::new(),
            creds,
        })
    }

    pub async fn get<T: DeserializeOwned>(&mut self, path: &str) -> Result<T> {
        let res = self.raw_request(Method::GET, path, None).await?;
        if res.status() == 401 {
            self.refresh().await?;
            return self
                .raw_request(Method::GET, path, None)
                .await?
                .json()
                .await
                .context("Failed to parse response");
        }
        self.handle_response(res).await
    }

    pub async fn post<T: DeserializeOwned>(&mut self, path: &str, body: Value) -> Result<T> {
        let res = self.raw_request(Method::POST, path, Some(body.clone())).await?;
        if res.status() == 401 {
            self.refresh().await?;
            return self
                .raw_request(Method::POST, path, Some(body))
                .await?
                .json()
                .await
                .context("Failed to parse response");
        }
        self.handle_response(res).await
    }

    pub async fn delete(&mut self, path: &str) -> Result<()> {
        let res = self.raw_request(Method::DELETE, path, None).await?;
        if res.status() == 401 {
            self.refresh().await?;
            let res = self.raw_request(Method::DELETE, path, None).await?;
            if !res.status().is_success() {
                bail!("Delete failed: {}", res.status());
            }
            return Ok(());
        }
        if !res.status().is_success() {
            let err: Value = res.json().await.unwrap_or_default();
            bail!("{}", err["message"].as_str().unwrap_or("Request failed"));
        }
        Ok(())
    }

    pub async fn get_bytes(&mut self, path: &str) -> Result<bytes::Bytes> {
        let res = self.raw_request(Method::GET, path, None).await?;
        if !res.status().is_success() {
            bail!("Request failed: {}", res.status());
        }
        Ok(res.bytes().await?)
    }

    async fn raw_request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", base_url(), path);
        let mut req = self
            .client
            .request(method, &url)
            .bearer_auth(&self.creds.access_token)
            .header("X-API-Version", "1");

        if let Some(b) = body {
            req = req.json(&b);
        }

        req.send().await.context("Network error")
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        res: reqwest::Response,
    ) -> Result<T> {
        if !res.status().is_success() {
            let err: Value = res.json().await.unwrap_or_default();
            bail!("{}", err["message"].as_str().unwrap_or("Request failed"));
        }
        res.json().await.context("Failed to parse response")
    }

    async fn refresh(&mut self) -> Result<()> {
        let url = format!("{}/auth/refresh", base_url());
        let res = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "refresh_token": self.creds.refresh_token }))
            .send()
            .await
            .context("Network error during token refresh")?;

        if !res.status().is_success() {
            bail!("Session expired. Please run `insighta login`.");
        }

        let data: Value = res.json().await?;
        let new_creds = Credentials {
            access_token: data["access_token"]
                .as_str()
                .context("Missing access_token")?
                .to_string(),
            refresh_token: data["refresh_token"]
                .as_str()
                .context("Missing refresh_token")?
                .to_string(),
        };

        config::save(&new_creds)?;
        self.creds = new_creds;
        Ok(())
    }
}