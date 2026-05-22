//! Official Rust SDK for the Orizn Visa API.
//!
//! Check visa requirements for 39,585 passport-destination pairs in 15 languages.
//!
//! # Quick start
//! ```rust,no_run
//! use orizn::Orizn;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), orizn::Error> {
//!     let client = Orizn::new();
//!     let result = client.check("FRA", "JPN").await?;
//!     println!("{}: {} days", result.requirement, result.visa_free_days.unwrap_or(0));
//!     Ok(())
//! }
//! ```

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API key required. Get one free at https://visa.orizn.app")]
    AuthRequired,
    #[error("Rate limit exceeded. Upgrade at https://visa.orizn.app")]
    RateLimit,
    #[error("No visa data for {passport} → {destination}")]
    NotFound { passport: String, destination: String },
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisaCheckResult {
    pub passport: String,
    pub destination: String,
    pub requirement: String,
    pub visa_free_days: Option<i32>,
    pub visa_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryInfo {
    pub currency: String,
    pub language: String,
    pub timezone: String,
    pub capital: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisaData {
    pub passport: String,
    pub destination: String,
    pub requirement: String,
    pub visa_free_days: Option<i32>,
    pub visa_required: bool,
    pub description: String,
    pub documents_required: Vec<String>,
    pub process: Vec<String>,
    pub tips: Vec<String>,
    pub country_info: CountryInfo,
    pub verified: bool,
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    data: T,
}

pub struct Orizn {
    client: Client,
    api_key: Option<String>,
    base_url: String,
}

impl Orizn {
    /// Create a new client. Reads ORIZN_API_KEY from environment.
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("orizn-rs/1.0.0")
                .build()
                .expect("Failed to build HTTP client"),
            api_key: env::var("ORIZN_API_KEY").ok(),
            base_url: "https://visa.orizn.app".to_string(),
        }
    }

    /// Create with explicit API key.
    pub fn with_key(api_key: impl Into<String>) -> Self {
        let mut c = Self::new();
        c.api_key = Some(api_key.into());
        c
    }

    /// Quick visa check. No API key needed.
    pub async fn check(&self, passport: &str, destination: &str) -> Result<VisaCheckResult, Error> {
        let resp = self.client
            .get(format!("{}/api/v1/visa/check", self.base_url))
            .query(&[
                ("passport", &passport.to_uppercase() as &str),
                ("destination", &destination.to_uppercase() as &str),
            ])
            .send()
            .await?;

        self.handle_status(&resp, passport, destination)?;
        let body: ApiResponse<VisaCheckResult> = resp.json().await?;
        Ok(body.data)
    }

    /// Full visa details. Requires API key.
    pub async fn get_visa(
        &self,
        passport: &str,
        destination: &str,
        lang: &str,
    ) -> Result<VisaData, Error> {
        let key = self.api_key.as_ref().ok_or(Error::AuthRequired)?;

        let resp = self.client
            .get(format!("{}/api/v1/visa", self.base_url))
            .header("x-api-key", key)
            .query(&[
                ("passport", &passport.to_uppercase() as &str),
                ("destination", &destination.to_uppercase() as &str),
                ("lang", lang),
            ])
            .send()
            .await?;

        self.handle_status(&resp, passport, destination)?;
        let body: ApiResponse<VisaData> = resp.json().await?;
        Ok(body.data)
    }

    fn handle_status(
        &self,
        resp: &reqwest::Response,
        passport: &str,
        destination: &str,
    ) -> Result<(), Error> {
        match resp.status().as_u16() {
            200..=299 => Ok(()),
            401 | 403 => Err(Error::AuthRequired),
            429 => Err(Error::RateLimit),
            404 => Err(Error::NotFound {
                passport: passport.to_string(),
                destination: destination.to_string(),
            }),
            s => Err(Error::Api {
                status: s,
                message: format!("Unexpected status {s}"),
            }),
        }
    }
}

impl Default for Orizn {
    fn default() -> Self {
        Self::new()
    }
}
