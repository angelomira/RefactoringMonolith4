/// Application configuration module
use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub nasa_api_url: String,
    pub nasa_api_key: String,
    pub where_iss_url: String,
    pub fetch_intervals: FetchIntervals,
}

#[derive(Clone, Debug)]
pub struct FetchIntervals {
    pub osdr_seconds: u64,
    pub iss_seconds: u64,
    pub apod_seconds: u64,
    pub neo_seconds: u64,
    pub donki_seconds: u64,
    pub spacex_seconds: u64,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is required");

        let nasa_api_url = env::var("NASA_API_URL").unwrap_or_else(|_| {
            "https://visualization.osdr.nasa.gov/biodata/api/v2/datasets/?format=json".to_string()
        });

        let nasa_api_key = env::var("NASA_API_KEY").unwrap_or_default();

        let where_iss_url = env::var("WHERE_ISS_URL")
            .unwrap_or_else(|_| "https://api.wheretheiss.at/v1/satellites/25544".to_string());

        let fetch_intervals = FetchIntervals {
            osdr_seconds: env_u64("FETCH_EVERY_SECONDS", 600),
            iss_seconds: env_u64("ISS_EVERY_SECONDS", 120),
            apod_seconds: env_u64("APOD_EVERY_SECONDS", 43200), // 12h
            neo_seconds: env_u64("NEO_EVERY_SECONDS", 7200),    // 2h
            donki_seconds: env_u64("DONKI_EVERY_SECONDS", 3600), // 1h
            spacex_seconds: env_u64("SPACEX_EVERY_SECONDS", 3600),
        };

        Ok(Self {
            database_url,
            nasa_api_url,
            nasa_api_key,
            where_iss_url,
            fetch_intervals,
        })
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}
