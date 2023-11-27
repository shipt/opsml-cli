/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use crate::api::types;
use crate::api::utils;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use owo_colors::OwoColorize;
use reqwest::{self, Response};
use serde_json;
use std::{format, fs, path::Path};
use tokio;

const MODEL_METADATA_FILE: &str = "metadata.json";
const NO_ONNX_URI: &str = "No onnx model uri found but onnx flag set to true";

pub struct ModelDownloader<'a> {
    pub name: Option<&'a str>,
    pub version: Option<&'a str>,
    pub uid: Option<&'a str>,
    pub write_dir: &'a str,
    pub ignore_release_candidates: &'a bool,
    pub onnx: &'a bool,
    pub no_onnx: &'a bool,
}

impl ModelDownloader<'_> {
    /// Create parent directories associated with path
    ///
    /// # Arguments
    ///
    /// * `path` - path to create
    ///
    fn create_dir_path(&self, path: &str) -> Result<(), anyhow::Error> {
        let path = std::path::Path::new(path);
        let prefix = path
            .parent()
            .with_context(|| "Failed to get parent directory")?;
        std::fs::create_dir_all(prefix)
            .with_context(|| format!("Failed to create directory path for {:?}", prefix))?;

        Ok(())
    }

    /// Parses stream response
    ///
    /// # Arguments
    ///
    /// * `response` - Response object
    ///
    /// # Returns
    /// * `String` - String representation of response
    ///
    async fn load_stream_response(&self, response: Response) -> Result<String, anyhow::Error> {
        let mut response_stream = response.bytes_stream();
        let mut stream_buffer = String::new();
        while let Some(item) = response_stream.next().await {
            let chunk = item.with_context(|| "failed to read stream response")?;
            let string_chunk = std::str::from_utf8(&chunk).unwrap();

            stream_buffer.push_str(string_chunk);
        }
        Ok(stream_buffer)
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
    async fn download_stream_to_file(
        &self,
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

    /// Saves metadata to json
    ///
    /// # Arguments
    ///
    /// * `metadata` - metadata to save
    /// * `path` - path to save to
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    ///
    async fn save_metadata_to_json(
        &self,
        metadata: &types::ModelMetadata,
        path: &str,
    ) -> Result<(), anyhow::Error> {
        let json_string =
            serde_json::to_string(metadata).with_context(|| "Failed to serialize metadata")?;
        fs::File::create(path).with_context(|| "Unable to create metadata file")?;
        fs::write(path, json_string).with_context(|| "Unable to write metadata file")?;
        Ok(())
    }

    /// Main function for downloading model metadata
    ///
    /// # Arguments
    ///
    /// * `args` - DownloadArgs struct
    ///
    /// # Returns
    /// * `Result<types::ModelMetadata, String>` - Result of model metadata download
    ///
    async fn get_model_metadata(&self) -> Result<types::ModelMetadata, anyhow::Error> {
        let save_path: String = format!("{}/{}", self.write_dir, MODEL_METADATA_FILE);

        let model_metadata_request = types::ModelMetadataRequest {
            name: self.name,
            version: self.version,
            uid: self.uid,
            ignore_release_candidates: self.ignore_release_candidates,
        };

        let response = utils::make_post_request(
            &utils::OpsmlPaths::MetadataDownload.as_str(),
            &model_metadata_request,
        )
        .await?;

        let loaded_response = self.load_stream_response(response).await?;
        let model_metadata: types::ModelMetadata = serde_json::from_str(&loaded_response)
            .with_context(|| "Failed to parse model Metadata")?;

        // create save path for metadata
        self.create_dir_path(&save_path)?;
        self.save_metadata_to_json(&model_metadata, &save_path)
            .await?;

        Ok(model_metadata)
    }

    /// Sets model uri (onnx or trained model) depending on boolean
    ///
    /// # Arguments
    ///
    /// * `onnx` - Flag to download onnx model
    /// * `model_metadata` - Model metadata
    ///
    /// # Returns
    /// * `(String, String)` - Tuple of filename and uri
    ///
    fn get_uri(
        &self,
        download_onnx: bool,
        model_metadata: &types::ModelMetadata,
    ) -> (String, String) {
        let uri = if download_onnx {
            model_metadata.onnx_uri.clone().expect(NO_ONNX_URI)
        } else {
            model_metadata.model_uri.clone()
        };

        let filepath = std::path::Path::new(&uri);
        let filename = filepath.file_name().unwrap().to_str().unwrap().to_string();

        (filename, uri)
    }

    /// Downloads metadata
    ///
    /// # Arguments
    ///
    /// * `args` - DownloadArgs struct
    ///
    async fn get_metadata(&self) -> Result<types::ModelMetadata, anyhow::Error> {
        // check args first
        utils::check_args(self.name, self.version, self.uid)
            .await
            .unwrap();
        let model_metadata = self.get_model_metadata().await?;

        Ok(model_metadata)
    }

    /// Downloads a model file
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
    async fn download_file(
        &self,
        url: &str,
        uri: &str,
        local_save_path: &str,
    ) -> Result<(), anyhow::Error> {
        let model_url = format!("{}?read_path={}", url, uri);
        let response = utils::make_get_request(&model_url).await?;
        let filepath = Path::new(local_save_path);
        self.download_stream_to_file(response, filepath).await?;

        Ok(())
    }

    /// Downloads a model file
    async fn download_model(&self) -> Result<(), anyhow::Error> {
        let download_onnx = !(self.no_onnx); //if no_onnx is true, download_onnx is false
        let model_metadata = self.get_metadata().await?;
        let (filename, model_uri) = self.get_uri(download_onnx, &model_metadata);

        println!("Downloading model: {}, {}", filename.green(), model_uri);

        let local_save_path = format!("{}/{}", self.write_dir, filename);

        // Create all parent dirs if not exist
        self.create_dir_path(&local_save_path)?;

        // Download model
        self.download_file(
            &utils::OpsmlPaths::Download.as_str(),
            &model_uri,
            &local_save_path,
        )
        .await?;

        Ok(())
    }
}

/// Downloads model metadata
///
/// * `name` - Name of model
/// * `team` - Team associated with model
/// * `version` - Version of model
/// * `uid` - uid of model
/// * `url` - url of opsml server
#[tokio::main]
pub async fn download_model_metadata(
    name: Option<&str>,
    version: Option<&str>,
    uid: Option<&str>,
    write_dir: &str,
    ignore_release_candidates: &bool,
) -> Result<types::ModelMetadata, anyhow::Error> {
    // check args first

    let model_downloader = ModelDownloader {
        name,
        version,
        uid,
        write_dir,
        ignore_release_candidates,
        onnx: &false,
        no_onnx: &false,
    };
    model_downloader.get_metadata().await
}

/// Downloads model file
///
/// * `name` - Name of model
/// * `team` - Team associated with model
/// * `version` - Version of model
/// * `uid` - uid of model
/// * `url` - url of opsml server
/// * `write_dir` - directory to write to
/// * `no_onnx` - Flag to not download onnx model
/// * `onnx` - Flag to download onnx model
///
#[tokio::main]
pub async fn download_model(
    name: Option<&str>,
    version: Option<&str>,
    uid: Option<&str>,
    write_dir: &str,
    no_onnx: &bool,
    onnx: &bool,
    ignore_release_candidates: &bool,
) -> Result<(), anyhow::Error> {
    let model_downloader = ModelDownloader {
        name,
        version,
        uid,
        write_dir,
        ignore_release_candidates,
        onnx,
        no_onnx,
    };
    model_downloader.download_model().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use std::env;
    use std::fs;
    use tokio;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_download_metadata() {
        let mut server = mockito::Server::new();
        let url = server.url();

        env::set_var("OPSML_TRACKING_URI", url);
        // read mock response object
        let path = "./src/api/test_utils/metadata_non_onnx.json";
        let data = fs::read_to_string(path).expect("Unable to read file");
        let mock_metadata: types::ModelMetadata = serde_json::from_str(&data).unwrap();

        let unique_name = Uuid::new_v4().to_string();
        let new_path = format!("./src/api/test_utils/{}_mock_response.json", unique_name);

        // Create a mock server
        let mock = server
            .mock("POST", "/opsml/models/metadata")
            .with_status(201)
            .with_body(data)
            .create();

        let downloader = ModelDownloader {
            name: Some("name"),
            version: Some("version"),
            uid: None,
            write_dir: &new_path,
            ignore_release_candidates: &false,
            onnx: &false,
            no_onnx: &false,
        };
        let model_metadata = downloader.get_metadata().await.unwrap();

        // assert structs are the same
        assert_json_eq!(mock_metadata, model_metadata);

        mock.assert();
        // clean up
        fs::remove_dir_all(&new_path).unwrap();
    }

    #[tokio::test]
    async fn test_download_model() {
        let mut server = mockito::Server::new();
        let url = server.url();

        env::set_var("OPSML_TRACKING_URI", url);
        // read mock response object
        let path = "./src/api/test_utils/metadata_non_onnx.json";
        let data = fs::read_to_string(path).expect("Unable to read file");

        let new_dir = format!("./src/api/test_utils/{}", Uuid::new_v4());

        // mock metadata
        let mock_metadata_path = server
            .mock("POST", "/opsml/models/metadata")
            .with_status(201)
            .with_body(&data)
            .create();

        // mock model

        let get_path = "/opsml/files/download?read_path=model.pkl";
        let mock_model_path = server
            .mock("GET", get_path)
            .with_status(201)
            .with_body(&data)
            .create();

        let downloader = ModelDownloader {
            name: Some("name"),
            version: Some("version"),
            uid: None,
            write_dir: &new_dir,
            ignore_release_candidates: &false,
            onnx: &false,
            no_onnx: &true,
        };

        downloader.download_model().await.unwrap();

        mock_model_path.assert();
        mock_metadata_path.assert();

        // clean up
        fs::remove_dir_all(&new_dir).unwrap();
    }
}
