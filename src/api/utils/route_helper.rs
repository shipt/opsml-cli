use crate::api::types;
use crate::api::utils::utils;
use anyhow::Context;
use futures_util::StreamExt;

use owo_colors::OwoColorize;

use reqwest::{self, Response};
use serde::Serialize;

use std::{format, path::Path};

struct RouteHelper {}

impl RouteHelper {
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
        let (client, parsed_url) = utils::create_client(url).await.unwrap();
        let msg = client.post(parsed_url).json(payload).send();

        match msg.await {
            Ok(response) => Ok(response),
            Err(e) => Err(anyhow::Error::msg(format!(
                "Failed to make post request: {}",
                e
            ))),
        }
    }

    /// async get request for metadata
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice
    ///
    pub async fn make_get_request(url: &str) -> Result<Response, anyhow::Error> {
        let (client, parsed_url) = utils::create_client(url).await.unwrap();
        let msg = client.get(parsed_url).send();

        match msg.await {
            Ok(response) => Ok(response),
            Err(e) => Err(anyhow::Error::msg(format!(
                "Failed to make get request: {}",
                e
            ))),
        }
    }

    /// Lists files associated with a model
    ///
    /// # Arguments
    ///
    /// * `rpath` - Remote path to file
    ///
    /// # Returns
    /// * `Result<types::ListFileResponse, String>` - Result of file download
    ///
    pub async fn list_files(rpath: &Path) -> Result<types::ListFileResponse, anyhow::Error> {
        let file_url = format!(
            "{}?path={}",
            utils::OpsmlPaths::ListFile.as_str(),
            rpath.to_str().unwrap()
        );
        let response = RouteHelper::make_get_request(&file_url).await?;
        let files = response.json::<types::ListFileResponse>().await?;
        Ok(files)
    }

    /// Downloads a stream to a file
    ///
    /// # Arguments
    ///
    /// * `response` - Response object
    /// * `filename` - Path to save file to
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    ///
    pub async fn download_stream_to_file(
        response: Response,
        filename: &Path,
    ) -> Result<(), anyhow::Error> {
        let mut response_stream = response.bytes_stream();
        let mut file = tokio::fs::File::create(filename).await.unwrap();

        while let Some(item) = response_stream.next().await {
            let chunk =
                item.with_context(|| format!("failed to read response for {:?}", filename))?;
            tokio::io::copy(&mut chunk.as_ref(), &mut file)
                .await
                .with_context(|| format!("failed to write response for {:?}", filename))?;
        }
        Ok(())
    }

    /// Downloads an artifact file
    ///
    /// # Arguments
    ///
    /// * `url` - url of opsml server
    /// * `uri` - uri of model
    /// * `local_save_path` - path to save model to
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    ///
    pub async fn download_file(lpath: &Path, rpath: &str) -> Result<(), anyhow::Error> {
        let filename = lpath.file_name().unwrap().to_str().unwrap().to_string();
        let model_url = format!("{}?path={}", utils::OpsmlPaths::Download.as_str(), rpath);
        let response = RouteHelper::make_get_request(&model_url).await?;

        if response.status().is_success() {
            println!("Downloading model: {}, {}", filename.green(), model_url);
            RouteHelper::download_stream_to_file(response, lpath).await?;
        } else {
            let error_message = format!(
                "Failed to download model: {}",
                response.text().await.unwrap().red()
            );
            return Err(anyhow::anyhow!(error_message));
        }

        Ok(())
    }
}
