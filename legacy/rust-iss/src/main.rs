/// Main application entry point with clean architecture
mod clients;
mod config;
mod domain;
mod errors;
mod handlers;
mod repo;
mod routes;
mod services;
mod utils;

use crate::clients::{IssClient, NasaClient, OsdrClient, SpaceXClient};
use crate::config::AppConfig;
use crate::handlers::AppState;
use crate::repo::{init_db, CacheRepo, IssRepo, OsdrRepo};
use crate::routes::build_router;
use crate::services::{IssService, OsdrService, SpaceService};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // Load configuration
    let config = AppConfig::from_env()?;
    info!("Configuration loaded successfully");

    // Initialize database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;
    info!("Database connection pool established");

    // Initialize database schema
    init_db(&pool).await?;
    info!("Database schema initialized");

    // Initialize repositories
    let iss_repo = IssRepo::new(pool.clone());
    let osdr_repo = OsdrRepo::new(pool.clone());
    let cache_repo = CacheRepo::new(pool.clone());

    // Initialize clients
    let iss_client = IssClient::new(config.where_iss_url.clone())?;
    let osdr_client = OsdrClient::new(config.nasa_api_url.clone())?;
    let nasa_client = NasaClient::new(config.nasa_api_key.clone())?;
    let spacex_client = SpaceXClient::new()?;

    // Initialize services
    let iss_service = Arc::new(IssService::new(iss_repo.clone(), iss_client));
    let osdr_service = Arc::new(OsdrService::new(osdr_repo.clone(), osdr_client));
    let space_service = Arc::new(SpaceService::new(
        cache_repo.clone(),
        iss_repo.clone(),
        osdr_repo.clone(),
        nasa_client,
        spacex_client,
    ));

    // Initialize application state
    let state = AppState {
        iss_service: iss_service.clone(),
        osdr_service: osdr_service.clone(),
        space_service: space_service.clone(),
    };

    // Start background tasks
    start_background_tasks(config.clone(), iss_service, osdr_service, space_service);

    // Build router
    let app = build_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    info!("rust_iss service listening on 0.0.0.0:3000");

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// Start all background data fetching tasks
fn start_background_tasks(
    config: AppConfig,
    iss_service: Arc<IssService>,
    osdr_service: Arc<OsdrService>,
    space_service: Arc<SpaceService>,
) {
    let intervals = config.fetch_intervals;

    // Background task: OSDR sync
    {
        let service = osdr_service.clone();
        let interval = intervals.osdr_seconds;
        tokio::spawn(async move {
            info!("Starting OSDR background task (interval: {}s)", interval);
            loop {
                if let Err(e) = service.sync().await {
                    error!("OSDR sync error: {:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    // Background task: ISS position tracking
    {
        let service = iss_service.clone();
        let interval = intervals.iss_seconds;
        tokio::spawn(async move {
            info!("Starting ISS tracking task (interval: {}s)", interval);
            loop {
                if let Err(e) = service.fetch_and_store().await {
                    error!("ISS fetch error: {:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    // Background task: APOD
    {
        let service = space_service.clone();
        let interval = intervals.apod_seconds;
        tokio::spawn(async move {
            info!("Starting APOD background task (interval: {}s)", interval);
            loop {
                if let Err(e) = service.fetch_apod().await {
                    error!("APOD fetch error: {:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    // Background task: NEO feed
    {
        let service = space_service.clone();
        let interval = intervals.neo_seconds;
        tokio::spawn(async move {
            info!("Starting NEO feed task (interval: {}s)", interval);
            loop {
                if let Err(e) = service.fetch_neo().await {
                    error!("NEO fetch error: {:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    // Background task: DONKI (FLR + CME)
    {
        let service = space_service.clone();
        let interval = intervals.donki_seconds;
        tokio::spawn(async move {
            info!("Starting DONKI background task (interval: {}s)", interval);
            loop {
                if let Err(e) = service.fetch_flr().await {
                    error!("DONKI FLR fetch error: {:?}", e);
                }
                if let Err(e) = service.fetch_cme().await {
                    error!("DONKI CME fetch error: {:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    // Background task: SpaceX launches
    {
        let service = space_service.clone();
        let interval = intervals.spacex_seconds;
        tokio::spawn(async move {
            info!("Starting SpaceX launches task (interval: {}s)", interval);
            loop {
                if let Err(e) = service.fetch_spacex().await {
                    error!("SpaceX fetch error: {:?}", e);
                }
                tokio::time::sleep(Duration::from_secs(interval)).await;
            }
        });
    }

    info!("All background tasks started successfully");
}
