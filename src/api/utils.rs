use anyhow::Context;
use lazy_static::lazy_static;
use owo_colors::OwoColorize;
use reqwest::Url;
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

        Err(_e) => {
            panic!(
                "{}",
                "No OPSML_TRACKING_URI found. Check your environment"
                    .bold()
                    .red()
            )
        }
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
) -> Result<(), anyhow::Error> {
    let common_args = [name, version];
    let has_common = common_args.iter().all(|i| i.is_none());

    let has_uid = uid.is_none();

    if has_common != has_uid {
        Ok(())
    } else {
        Err(anyhow::Error::msg(
            "Please provide either a uid or a name and version",
        ))
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

pub async fn create_client(url: &str) -> Result<(reqwest::Client, Url), anyhow::Error> {
    let parsed_url = reqwest::Url::parse(url).with_context(|| "Failed to parse url")?;
    let client = reqwest::Client::new();

    Ok((client, parsed_url))
}

/// async post request for metadata
///
/// # Arguments
///
/// * `url` - A string slice
/// * `payload` - A string slice
///
pub async fn make_post_request<T: Serialize>(
    url: &str,
    payload: &T,
) -> Result<Response, anyhow::Error> {
    let (client, parsed_url) = create_client(url).await.unwrap();
    let msg = client.post(parsed_url).json(payload).send();

    match msg.await {
        Ok(response) => Ok(response),
        Err(e) => Err(anyhow::Error::msg(format!(
            "Failed to make post request: {}",
            e
        ))),
    }
}

pub async fn make_get_request(url: &str) -> Result<Response, anyhow::Error> {
    let (client, parsed_url) = create_client(url).await.unwrap();
    let msg = client.get(parsed_url).send();

    match msg.await {
        Ok(response) => Ok(response),
        Err(e) => Err(anyhow::Error::msg(format!(
            "Failed to make get request: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_suffix() {
        let test_uri_with_slash = "http://localhost:8080/";
        let test_uri_without_slash = "http://localhost:8080";
        let processed_with_slash_uri = remove_suffix(test_uri_with_slash, '/');
        let processed_without_slash_uri = remove_suffix(test_uri_without_slash, '/');
        assert_eq!(processed_with_slash_uri, "http://localhost:8080");
        assert_eq!(processed_without_slash_uri, test_uri_without_slash);
    }
}
