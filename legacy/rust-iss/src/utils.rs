/// Utility functions
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;

/// Extract number from JSON value
pub fn num(v: &Value) -> Option<f64> {
    if let Some(x) = v.as_f64() {
        return Some(x);
    }
    if let Some(s) = v.as_str() {
        return s.parse::<f64>().ok();
    }
    None
}

/// Calculate distance between two coordinates using Haversine formula
pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let rlat1 = lat1.to_radians();
    let rlat2 = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + rlat1.cos() * rlat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    6371.0 * c
}

/// Pick string value from JSON by trying multiple keys
pub fn s_pick(v: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(s) = x.as_str() {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            } else if x.is_number() {
                return Some(x.to_string());
            }
        }
    }
    None
}

/// Pick timestamp value from JSON by trying multiple keys
pub fn t_pick(v: &Value, keys: &[&str]) -> Option<DateTime<Utc>> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(s) = x.as_str() {
                if let Ok(dt) = s.parse::<DateTime<Utc>>() {
                    return Some(dt);
                }
                if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                    return Some(Utc.from_utc_datetime(&ndt));
                }
            } else if let Some(n) = x.as_i64() {
                return Some(Utc.timestamp_opt(n, 0).single().unwrap_or_else(Utc::now));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_km_zero_distance() {
        let distance = haversine_km(0.0, 0.0, 0.0, 0.0);
        assert_eq!(distance, 0.0);
    }

    #[test]
    fn test_haversine_km_known_distance() {
        // London (51.5074°N, 0.1278°W) to Paris (48.8566°N, 2.3522°E)
        // Approximate distance: ~344 km
        let distance = haversine_km(51.5074, -0.1278, 48.8566, 2.3522);
        assert!((distance - 344.0).abs() < 10.0); // Within 10km tolerance
    }

    #[test]
    fn test_num_from_float() {
        let json = serde_json::json!(42.5);
        assert_eq!(num(&json), Some(42.5));
    }

    #[test]
    fn test_num_from_string() {
        let json = serde_json::json!("42.5");
        assert_eq!(num(&json), Some(42.5));
    }

    #[test]
    fn test_num_from_invalid() {
        let json = serde_json::json!("invalid");
        assert_eq!(num(&json), None);
    }

    #[test]
    fn test_s_pick_finds_first() {
        let json = serde_json::json!({"name": "test", "title": "backup"});
        assert_eq!(s_pick(&json, &["name", "title"]), Some("test".to_string()));
    }

    #[test]
    fn test_s_pick_finds_second() {
        let json = serde_json::json!({"title": "backup"});
        assert_eq!(
            s_pick(&json, &["name", "title"]),
            Some("backup".to_string())
        );
    }

    #[test]
    fn test_s_pick_not_found() {
        let json = serde_json::json!({"other": "value"});
        assert_eq!(s_pick(&json, &["name", "title"]), None);
    }

    #[test]
    fn test_t_pick_iso_format() {
        let json = serde_json::json!({"timestamp": "2024-01-15T10:30:00Z"});
        let result = t_pick(&json, &["timestamp"]);
        assert!(result.is_some());
    }

    #[test]
    fn test_t_pick_from_unix_timestamp() {
        let json = serde_json::json!({"timestamp": 1705315800});
        let result = t_pick(&json, &["timestamp"]);
        assert!(result.is_some());
    }

    #[test]
    fn test_t_pick_not_found() {
        let json = serde_json::json!({"other": "value"});
        assert_eq!(t_pick(&json, &["timestamp"]), None);
    }
}
