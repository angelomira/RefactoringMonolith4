/// HTTP request handlers
use crate::domain::{Health, IssTrend, SpaceSummary};
use crate::errors::ApiError;
use crate::services::{IssService, OsdrService, SpaceService};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub iss_service: Arc<IssService>,
    pub osdr_service: Arc<OsdrService>,
    pub space_service: Arc<SpaceService>,
}

/// Successful response wrapper
#[derive(Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub ok: bool,
    #[serde(flatten)]
    pub data: T,
}

impl<T: Serialize> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self { ok: true, data }
    }
}

/// Health check handler
pub async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        now: Utc::now(),
    })
}

/// Get latest ISS position
pub async fn get_last_iss(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let data = state.iss_service.get_latest().await?;

    match data {
        Some(iss) => Ok(Json(serde_json::json!(SuccessResponse::new(iss)))),
        None => Ok(Json(serde_json::json!(SuccessResponse::new(
            serde_json::json!({
                "message": "no data"
            })
        )))),
    }
}

/// Trigger ISS position fetch
pub async fn trigger_iss_fetch(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    state.iss_service.fetch_and_store().await?;
    get_last_iss(State(state)).await
}

/// Get ISS movement trend
pub async fn get_iss_trend(
    State(state): State<AppState>,
) -> Result<Json<SuccessResponse<IssTrend>>, ApiError> {
    let trend = state.iss_service.calculate_trend().await?;
    Ok(Json(SuccessResponse::new(trend)))
}

/// Sync OSDR datasets
pub async fn sync_osdr(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let written = state.osdr_service.sync().await?;
    Ok(Json(serde_json::json!(SuccessResponse::new(
        serde_json::json!({
            "written": written
        })
    ))))
}

/// List OSDR datasets
pub async fn list_osdr(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let limit = std::env::var("OSDR_LIST_LIMIT")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(20);

    let items = state.osdr_service.list(limit).await?;
    Ok(Json(serde_json::json!(SuccessResponse::new(
        serde_json::json!({
            "items": items
        })
    ))))
}

/// Get latest space data for a source
pub async fn get_space_latest(
    Path(source): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let data = state.space_service.get_latest(&source).await?;

    match data {
        Some(cache) => Ok(Json(serde_json::json!(SuccessResponse::new(cache)))),
        None => Ok(Json(serde_json::json!(SuccessResponse::new(
            serde_json::json!({
                "source": source,
                "message": "no data"
            })
        )))),
    }
}

/// Refresh space data sources
pub async fn refresh_space(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let sources_str = params
        .get("src")
        .cloned()
        .unwrap_or_else(|| "apod,neo,flr,cme,spacex".to_string());

    let sources: Vec<&str> = sources_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let refreshed = state.space_service.refresh(&sources).await?;

    Ok(Json(serde_json::json!(SuccessResponse::new(
        serde_json::json!({
            "refreshed": refreshed
        })
    ))))
}

/// Get space data summary
pub async fn get_space_summary(
    State(state): State<AppState>,
) -> Result<Json<SuccessResponse<SpaceSummary>>, ApiError> {
    let summary = state.space_service.get_summary().await?;
    Ok(Json(SuccessResponse::new(summary)))
}
