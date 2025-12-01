/// External API clients module
use crate::errors::ApiResult;
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

/// HTTP client wrapper with common configuration
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> ApiResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("rust-iss-service/1.0")
            .build()?;
        Ok(Self { client })
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

/// ISS tracking client
pub struct IssClient {
    http_client: HttpClient,
    base_url: String,
}

impl IssClient {
    pub fn new(base_url: String) -> ApiResult<Self> {
        Ok(Self {
            http_client: HttpClient::new()?,
            base_url,
        })
    }

    /// Get base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Fetch current ISS position
    pub async fn fetch_position(&self) -> ApiResult<Value> {
        let resp = self
            .http_client
            .get_client()
            .get(&self.base_url)
            .send()
            .await?;

        let json = resp.json().await?;
        Ok(json)
    }
}

/// NASA OSDR client
pub struct OsdrClient {
    http_client: HttpClient,
    base_url: String,
}

impl OsdrClient {
    pub fn new(base_url: String) -> ApiResult<Self> {
        Ok(Self {
            http_client: HttpClient::new()?,
            base_url,
        })
    }

    /// Fetch OSDR datasets
    pub async fn fetch_datasets(&self) -> ApiResult<Vec<Value>> {
        let resp = self
            .http_client
            .get_client()
            .get(&self.base_url)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(crate::errors::ApiError::Internal(format!(
                "OSDR request failed with status {}",
                resp.status()
            )));
        }

        let json: Value = resp.json().await?;

        // Handle different response formats
        let items = if let Some(arr) = json.as_array() {
            arr.clone()
        } else if let Some(arr) = json.get("items").and_then(|x| x.as_array()) {
            arr.clone()
        } else if let Some(arr) = json.get("results").and_then(|x| x.as_array()) {
            arr.clone()
        } else {
            vec![json]
        };

        Ok(items)
    }
}

/// NASA APIs client (APOD, NeoWs, DONKI)
pub struct NasaClient {
    http_client: HttpClient,
    api_key: String,
}

impl NasaClient {
    pub fn new(api_key: String) -> ApiResult<Self> {
        Ok(Self {
            http_client: HttpClient::new()?,
            api_key,
        })
    }

    /// Fetch Astronomy Picture of the Day
    pub async fn fetch_apod(&self) -> ApiResult<Value> {
        let url = "https://api.nasa.gov/planetary/apod";
        let mut req = self
            .http_client
            .get_client()
            .get(url)
            .query(&[("thumbs", "true")]);

        if !self.api_key.is_empty() {
            req = req.query(&[("api_key", &self.api_key)]);
        }

        let json = req.send().await?.json().await?;
        Ok(json)
    }

    /// Fetch Near Earth Objects feed
    pub async fn fetch_neo_feed(&self) -> ApiResult<Value> {
        let today = Utc::now().date_naive();
        let start = today - chrono::Days::new(2);
        let url = "https://api.nasa.gov/neo/rest/v1/feed";

        let mut req = self.http_client.get_client().get(url).query(&[
            ("start_date", start.to_string()),
            ("end_date", today.to_string()),
        ]);

        if !self.api_key.is_empty() {
            req = req.query(&[("api_key", &self.api_key)]);
        }

        let json = req.send().await?.json().await?;
        Ok(json)
    }

    /// Fetch DONKI Solar Flares
    pub async fn fetch_donki_flr(&self) -> ApiResult<Value> {
        let (from, to) = Self::last_days(5);
        let url = "https://api.nasa.gov/DONKI/FLR";

        let mut req = self
            .http_client
            .get_client()
            .get(url)
            .query(&[("startDate", from), ("endDate", to)]);

        if !self.api_key.is_empty() {
            req = req.query(&[("api_key", &self.api_key)]);
        }

        let json = req.send().await?.json().await?;
        Ok(json)
    }

    /// Fetch DONKI Coronal Mass Ejections
    pub async fn fetch_donki_cme(&self) -> ApiResult<Value> {
        let (from, to) = Self::last_days(5);
        let url = "https://api.nasa.gov/DONKI/CME";

        let mut req = self
            .http_client
            .get_client()
            .get(url)
            .query(&[("startDate", from), ("endDate", to)]);

        if !self.api_key.is_empty() {
            req = req.query(&[("api_key", &self.api_key)]);
        }

        let json = req.send().await?.json().await?;
        Ok(json)
    }

    fn last_days(n: i64) -> (String, String) {
        let to = Utc::now().date_naive();
        let from = to - chrono::Days::new(n as u64);
        (from.to_string(), to.to_string())
    }
}

/// SpaceX API client
pub struct SpaceXClient {
    http_client: HttpClient,
}

impl SpaceXClient {
    pub fn new() -> ApiResult<Self> {
        Ok(Self {
            http_client: HttpClient::new()?,
        })
    }

    /// Fetch next SpaceX launch
    pub async fn fetch_next_launch(&self) -> ApiResult<Value> {
        let url = "https://api.spacexdata.com/v4/launches/next";
        let json = self
            .http_client
            .get_client()
            .get(url)
            .send()
            .await?
            .json()
            .await?;
        Ok(json)
    }
}
