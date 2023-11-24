use lazy_static::lazy_static;
use reqwest::{self, Response};
use serde::Serialize;
use std::env;

lazy_static! {
    static ref OPSML_TRACKING_URI: String = match env::var("OPSML_TRACKING_URI") {
        Ok(val) =>
            if val.ends_with('/') {
                remove_suffix(&val, '/')
            } else {
                val
            },
        Err(_e) => panic!("No tracking uri set"),
    };
}

pub enum OpsmlPaths {
    ListCard,
    MetadataDownload,
    Download,
    Metric,
    CompareMetric,
}

impl OpsmlPaths {
    pub fn as_str(&self) -> String {
        match self {
            OpsmlPaths::ListCard => format!("{}/opsml/cards/list", *OPSML_TRACKING_URI),
            OpsmlPaths::MetadataDownload => {
                format!("{}/opsml/models/metadata", *OPSML_TRACKING_URI)
            }
            OpsmlPaths::Download => {
                format!("{}/opsml/files/download", *OPSML_TRACKING_URI)
            }
            OpsmlPaths::Metric => {
                format!("{}/opsml/models/metrics", *OPSML_TRACKING_URI)
            }
            OpsmlPaths::CompareMetric => {
                format!("{}/opsml/models/compare_metrics", *OPSML_TRACKING_URI)
            }
        }
    }
}

pub async fn check_args(
    name: &Option<String>,
    version: &Option<String>,
    uid: &Option<String>,
) -> Result<(), String> {
    let common_args = [name, version];
    let has_common = common_args.iter().all(|i| i.is_none());

    let has_uid = uid.is_none();

    if has_common != has_uid {
        Ok(())
    } else {
        Err("Either name/version or uid must be specified".to_string())
    }
}

/// Removes the suffix from a string if it exists
///
/// # Arguments
///
/// * `s` - A string slice
/// * `suffix` - A string slice
///
pub fn remove_suffix(s: &str, suffix: char) -> String {
    match s.strip_suffix(suffix) {
        Some(s) => s.to_string(),
        None => s.to_string(),
    }
}

/// async post request for metadata
///
/// # Arguments
///
/// * `url` - A string slice
/// * `payload` - A string slice
///
pub async fn make_post_request<T: Serialize>(url: &str, payload: &T) -> Response {
    let parsed_url = reqwest::Url::parse(url).unwrap();
    let client = reqwest::Client::new();

    client.post(parsed_url).json(payload).send().await.unwrap()
}

pub async fn make_get_request(url: &str) -> Response {
    let parsed_url = reqwest::Url::parse(url).unwrap();
    let client = reqwest::Client::new();

    client.get(parsed_url).send().await.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_suffix() {
        let test_uri_with_slash = "http://localhost:8080/";
        let test_uri_without_slash = "http://localhost:8080";
        let processed_with_slash_uri = remove_suffix(&test_uri_with_slash, '/');
        let processed_without_slash_uri = remove_suffix(&test_uri_without_slash, '/');
        assert_eq!(processed_with_slash_uri, "http://localhost:8080");
        assert_eq!(processed_without_slash_uri, test_uri_without_slash);
    }
}
