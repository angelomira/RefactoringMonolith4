/// Business logic services layer
use crate::clients::{IssClient, NasaClient, OsdrClient, SpaceXClient};
use crate::domain::{IssTrend, SpaceSummary};
use crate::errors::ApiResult;
use crate::repo::{CacheRepo, IssRepo, OsdrRepo};
use crate::utils::{haversine_km, num, s_pick, t_pick};
use serde_json::Value;

/// ISS tracking service
pub struct IssService {
    repo: IssRepo,
    client: IssClient,
}

impl IssService {
    pub fn new(repo: IssRepo, client: IssClient) -> Self {
        Self { repo, client }
    }

    /// Fetch ISS position and store in database
    pub async fn fetch_and_store(&self) -> ApiResult<()> {
        let position = self.client.fetch_position().await?;
        self.repo
            .insert_log(self.client.base_url(), position)
            .await?;
        Ok(())
    }

    /// Get latest ISS position
    pub async fn get_latest(&self) -> ApiResult<Option<Value>> {
        let log = self.repo.get_latest().await?;
        Ok(log.map(|l| {
            serde_json::json!({
                "id": l.id,
                "fetched_at": l.fetched_at,
                "source_url": l.source_url,
                "payload": l.payload
            })
        }))
    }

    /// Calculate ISS movement trend
    pub async fn calculate_trend(&self) -> ApiResult<IssTrend> {
        let rows = self.repo.get_last_n(2).await?;

        if rows.len() < 2 {
            return Ok(IssTrend {
                movement: false,
                delta_km: 0.0,
                dt_sec: 0.0,
                velocity_kmh: None,
                from_time: None,
                to_time: None,
                from_lat: None,
                from_lon: None,
                to_lat: None,
                to_lon: None,
            });
        }

        let (t2, p2) = &rows[0];
        let (t1, p1) = &rows[1];

        let lat1 = num(&p1["latitude"]);
        let lon1 = num(&p1["longitude"]);
        let lat2 = num(&p2["latitude"]);
        let lon2 = num(&p2["longitude"]);
        let v2 = num(&p2["velocity"]);

        let mut delta_km = 0.0;
        let mut movement = false;
        if let (Some(a1), Some(o1), Some(a2), Some(o2)) = (lat1, lon1, lat2, lon2) {
            delta_km = haversine_km(a1, o1, a2, o2);
            movement = delta_km > 0.1;
        }
        let dt_sec = (*t2 - *t1).num_milliseconds() as f64 / 1000.0;

        Ok(IssTrend {
            movement,
            delta_km,
            dt_sec,
            velocity_kmh: v2,
            from_time: Some(*t1),
            to_time: Some(*t2),
            from_lat: lat1,
            from_lon: lon1,
            to_lat: lat2,
            to_lon: lon2,
        })
    }
}

/// OSDR data service
pub struct OsdrService {
    repo: OsdrRepo,
    client: OsdrClient,
}

impl OsdrService {
    pub fn new(repo: OsdrRepo, client: OsdrClient) -> Self {
        Self { repo, client }
    }

    /// Fetch OSDR datasets and store in database
    pub async fn sync(&self) -> ApiResult<usize> {
        let items = self.client.fetch_datasets().await?;

        let mut written = 0;
        for item in items {
            let dataset_id = s_pick(
                &item,
                &[
                    "dataset_id",
                    "id",
                    "uuid",
                    "studyId",
                    "accession",
                    "osdr_id",
                ],
            );
            let title = s_pick(&item, &["title", "name", "label"]);
            let status = s_pick(&item, &["status", "state", "lifecycle"]);
            let updated = t_pick(
                &item,
                &[
                    "updated",
                    "updated_at",
                    "modified",
                    "lastUpdated",
                    "timestamp",
                ],
            );

            self.repo
                .upsert_item(dataset_id, title, status, updated, item)
                .await?;
            written += 1;
        }

        Ok(written)
    }

    /// List OSDR items
    pub async fn list(&self, limit: i64) -> ApiResult<Vec<Value>> {
        let items = self.repo.list_items(limit).await?;

        let result = items
            .into_iter()
            .map(|item| {
                serde_json::json!({
                    "id": item.id,
                    "dataset_id": item.dataset_id,
                    "title": item.title,
                    "status": item.status,
                    "updated_at": item.updated_at,
                    "inserted_at": item.inserted_at,
                    "raw": item.raw,
                })
            })
            .collect();

        Ok(result)
    }
}

/// Space data aggregation service
pub struct SpaceService {
    cache_repo: CacheRepo,
    iss_repo: IssRepo,
    osdr_repo: OsdrRepo,
    nasa_client: NasaClient,
    spacex_client: SpaceXClient,
}

impl SpaceService {
    pub fn new(
        cache_repo: CacheRepo,
        iss_repo: IssRepo,
        osdr_repo: OsdrRepo,
        nasa_client: NasaClient,
        spacex_client: SpaceXClient,
    ) -> Self {
        Self {
            cache_repo,
            iss_repo,
            osdr_repo,
            nasa_client,
            spacex_client,
        }
    }

    /// Fetch and cache APOD
    pub async fn fetch_apod(&self) -> ApiResult<()> {
        let data = self.nasa_client.fetch_apod().await?;
        self.cache_repo.write("apod", data).await
    }

    /// Fetch and cache NEO data
    pub async fn fetch_neo(&self) -> ApiResult<()> {
        let data = self.nasa_client.fetch_neo_feed().await?;
        self.cache_repo.write("neo", data).await
    }

    /// Fetch and cache DONKI FLR
    pub async fn fetch_flr(&self) -> ApiResult<()> {
        let data = self.nasa_client.fetch_donki_flr().await?;
        self.cache_repo.write("flr", data).await
    }

    /// Fetch and cache DONKI CME
    pub async fn fetch_cme(&self) -> ApiResult<()> {
        let data = self.nasa_client.fetch_donki_cme().await?;
        self.cache_repo.write("cme", data).await
    }

    /// Fetch and cache SpaceX next launch
    pub async fn fetch_spacex(&self) -> ApiResult<()> {
        let data = self.spacex_client.fetch_next_launch().await?;
        self.cache_repo.write("spacex", data).await
    }

    /// Get latest cached data for a source
    pub async fn get_latest(&self, source: &str) -> ApiResult<Option<Value>> {
        let cache = self.cache_repo.get_latest(source).await?;
        Ok(cache.map(|c| {
            serde_json::json!({
                "source": c.source,
                "fetched_at": c.fetched_at,
                "payload": c.payload
            })
        }))
    }

    /// Refresh multiple sources
    pub async fn refresh(&self, sources: &[&str]) -> ApiResult<Vec<String>> {
        let mut refreshed = Vec::new();

        for &source in sources {
            let result = match source {
                "apod" => self.fetch_apod().await,
                "neo" => self.fetch_neo().await,
                "flr" => self.fetch_flr().await,
                "cme" => self.fetch_cme().await,
                "spacex" => self.fetch_spacex().await,
                _ => continue,
            };

            if result.is_ok() {
                refreshed.push(source.to_string());
            }
        }

        Ok(refreshed)
    }

    /// Get summary of all space data
    pub async fn get_summary(&self) -> ApiResult<SpaceSummary> {
        let apod = self.get_latest_or_empty("apod").await;
        let neo = self.get_latest_or_empty("neo").await;
        let flr = self.get_latest_or_empty("flr").await;
        let cme = self.get_latest_or_empty("cme").await;
        let spacex = self.get_latest_or_empty("spacex").await;

        let iss = self.iss_repo.get_latest().await?;
        let iss_value = iss
            .map(|l| {
                serde_json::json!({
                    "at": l.fetched_at,
                    "payload": l.payload
                })
            })
            .unwrap_or_else(|| serde_json::json!({}));

        let osdr_count = self.osdr_repo.count_items().await?;

        Ok(SpaceSummary {
            apod,
            neo,
            flr,
            cme,
            spacex,
            iss: iss_value,
            osdr_count,
        })
    }

    async fn get_latest_or_empty(&self, source: &str) -> Value {
        self.cache_repo
            .get_latest(source)
            .await
            .ok()
            .flatten()
            .map(|c| {
                serde_json::json!({
                    "at": c.fetched_at,
                    "payload": c.payload
                })
            })
            .unwrap_or_else(|| serde_json::json!({}))
    }
}
