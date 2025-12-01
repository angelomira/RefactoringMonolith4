/// Repository layer for database operations
use crate::domain::{IssLog, OsdrItem, SpaceCache};
use crate::errors::ApiResult;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

/// ISS data repository
#[derive(Clone)]
pub struct IssRepo {
    pool: PgPool,
}

impl IssRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Insert new ISS fetch log
    pub async fn insert_log(&self, source_url: &str, payload: Value) -> ApiResult<()> {
        sqlx::query("INSERT INTO iss_fetch_log (source_url, payload) VALUES ($1, $2)")
            .bind(source_url)
            .bind(payload)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get the most recent ISS log
    pub async fn get_latest(&self) -> ApiResult<Option<IssLog>> {
        let row = sqlx::query_as::<_, (i64, DateTime<Utc>, String, Value)>(
            "SELECT id, fetched_at, source_url, payload
             FROM iss_fetch_log
             ORDER BY id DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, fetched_at, source_url, payload)| IssLog {
            id,
            fetched_at,
            source_url,
            payload,
        }))
    }

    /// Get last N ISS logs for trend analysis
    pub async fn get_last_n(&self, n: i64) -> ApiResult<Vec<(DateTime<Utc>, Value)>> {
        let rows = sqlx::query_as::<_, (DateTime<Utc>, Value)>(
            "SELECT fetched_at, payload FROM iss_fetch_log ORDER BY id DESC LIMIT $1",
        )
        .bind(n)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}

/// OSDR data repository
#[derive(Clone)]
pub struct OsdrRepo {
    pool: PgPool,
}

impl OsdrRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert OSDR item by dataset_id (business key)
    pub async fn upsert_item(
        &self,
        dataset_id: Option<String>,
        title: Option<String>,
        status: Option<String>,
        updated_at: Option<DateTime<Utc>>,
        raw: Value,
    ) -> ApiResult<()> {
        if let Some(ds_id) = dataset_id {
            // Upsert by business key (dataset_id)
            sqlx::query(
                "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                 VALUES($1,$2,$3,$4,$5)
                 ON CONFLICT (dataset_id) DO UPDATE
                 SET title=EXCLUDED.title, status=EXCLUDED.status,
                     updated_at=EXCLUDED.updated_at, raw=EXCLUDED.raw",
            )
            .bind(ds_id)
            .bind(title)
            .bind(status)
            .bind(updated_at)
            .bind(raw)
            .execute(&self.pool)
            .await?;
        } else {
            // Insert without conflict handling
            sqlx::query(
                "INSERT INTO osdr_items(dataset_id, title, status, updated_at, raw)
                 VALUES($1,$2,$3,$4,$5)",
            )
            .bind::<Option<String>>(None)
            .bind(title)
            .bind(status)
            .bind(updated_at)
            .bind(raw)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// List OSDR items with limit
    pub async fn list_items(&self, limit: i64) -> ApiResult<Vec<OsdrItem>> {
        let rows = sqlx::query_as::<
            _,
            (
                i64,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<DateTime<Utc>>,
                DateTime<Utc>,
                Value,
            ),
        >(
            "SELECT id, dataset_id, title, status, updated_at, inserted_at, raw
             FROM osdr_items
             ORDER BY inserted_at DESC
             LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(id, dataset_id, title, status, updated_at, inserted_at, raw)| OsdrItem {
                    id,
                    dataset_id,
                    title,
                    status,
                    updated_at,
                    inserted_at,
                    raw,
                },
            )
            .collect())
    }

    /// Count total OSDR items
    pub async fn count_items(&self) -> ApiResult<i64> {
        let row = sqlx::query_as::<_, (i64,)>("SELECT count(*) FROM osdr_items")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}

/// Space cache repository
#[derive(Clone)]
pub struct CacheRepo {
    pool: PgPool,
}

impl CacheRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Write data to cache
    pub async fn write(&self, source: &str, payload: Value) -> ApiResult<()> {
        sqlx::query("INSERT INTO space_cache(source, payload) VALUES ($1,$2)")
            .bind(source)
            .bind(payload)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get latest cache entry for a source
    pub async fn get_latest(&self, source: &str) -> ApiResult<Option<SpaceCache>> {
        let row = sqlx::query_as::<_, (i64, String, DateTime<Utc>, Value)>(
            "SELECT id, source, fetched_at, payload FROM space_cache
             WHERE source = $1 ORDER BY id DESC LIMIT 1",
        )
        .bind(source)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, source, fetched_at, payload)| SpaceCache {
            id,
            source,
            fetched_at,
            payload,
        }))
    }
}

/// Initialize database tables
pub async fn init_db(pool: &PgPool) -> ApiResult<()> {
    // ISS
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS iss_fetch_log(
            id BIGSERIAL PRIMARY KEY,
            fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            source_url TEXT NOT NULL,
            payload JSONB NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    // OSDR
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS osdr_items(
            id BIGSERIAL PRIMARY KEY,
            dataset_id TEXT,
            title TEXT,
            status TEXT,
            updated_at TIMESTAMPTZ,
            inserted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            raw JSONB NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS ux_osdr_dataset_id
         ON osdr_items(dataset_id) WHERE dataset_id IS NOT NULL",
    )
    .execute(pool)
    .await?;

    // Universal space data cache
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS space_cache(
            id BIGSERIAL PRIMARY KEY,
            source TEXT NOT NULL,
            fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
            payload JSONB NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS ix_space_cache_source 
         ON space_cache(source,fetched_at DESC)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
