/// Application routes configuration
use crate::handlers::{
    get_iss_trend, get_last_iss, get_space_latest, get_space_summary, health, list_osdr,
    refresh_space, sync_osdr, trigger_iss_fetch, AppState,
};
use axum::{routing::get, Router};

/// Build the application router with all routes
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health))
        // ISS endpoints
        .route("/last", get(get_last_iss))
        .route("/fetch", get(trigger_iss_fetch))
        .route("/iss/trend", get(get_iss_trend))
        // OSDR endpoints
        .route("/osdr/sync", get(sync_osdr))
        .route("/osdr/list", get(list_osdr))
        // Space cache endpoints
        .route("/space/:src/latest", get(get_space_latest))
        .route("/space/refresh", get(refresh_space))
        .route("/space/summary", get(get_space_summary))
        .with_state(state)
}
