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
use std::path::PathBuf;
use std::{format, fs, path::Path};
use tokio;

use super::utils::OpsmlPaths;

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
    pub quantize: &'a bool,
}

impl ModelDownloader<'_> {
    /// Create parent directories associated with path
    ///
    /// # Arguments
    ///
    /// * `path` - path to create
    ///
    fn create_dir_path(&self, path: &Path) -> Result<(), anyhow::Error> {
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
        path: &Path,
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
        let save_path = Path::new(&self.write_dir).join(MODEL_METADATA_FILE);

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
    /// * `&Path` - Remote path to file
    ///
    fn get_model_uri(&self, download_onnx: bool, model_metadata: &types::ModelMetadata) -> PathBuf {
        let uri = if download_onnx {
            if self.quantize == &true {
                model_metadata
                    .quantized_model_uri
                    .clone()
                    .expect(NO_ONNX_URI)
            } else {
                model_metadata.onnx_uri.clone().expect(NO_ONNX_URI)
            }
        } else {
            model_metadata.model_uri.clone()
        };

        let filepath = std::path::Path::new(&uri);

        filepath.to_owned()
    }

    /// Gets processor uri
    ///
    /// # Arguments
    ///
    /// * `model_metadata` - Model metadata
    ///
    /// # Returns
    /// * `Option<&Path>` - File path to processor or None
    ///
    fn get_preprocessor_uri(&self, model_metadata: &types::ModelMetadata) -> Option<PathBuf> {
        let uri = if model_metadata.preprocessor_uri.is_some() {
            Some(
                std::path::Path::new(&model_metadata.preprocessor_uri.as_ref().unwrap()).to_owned(),
            )
        } else if model_metadata.tokenizer_uri.is_some() {
            Some(std::path::Path::new(&model_metadata.tokenizer_uri.as_ref().unwrap()).to_owned())
        } else if model_metadata.feature_extractor_uri.is_some() {
            Some(
                std::path::Path::new(&model_metadata.feature_extractor_uri.as_ref().unwrap())
                    .to_owned(),
            )
        } else {
            None
        };

        uri.to_owned()
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
    async fn download_file(&self, lpath: &Path, rpath: &str) -> Result<(), anyhow::Error> {
        let filename = lpath.file_name().unwrap().to_str().unwrap().to_string();
        let model_url = format!("{}?path={}", OpsmlPaths::Download.as_str(), rpath);
        let response = utils::make_get_request(&model_url).await?;

        if response.status().is_success() {
            println!("Downloading model: {}, {}", filename.green(), model_url);
            self.download_stream_to_file(response, lpath).await?;
        } else {
            let error_message = format!(
                "Failed to download model: {}",
                response.text().await.unwrap().red()
            );
            return Err(anyhow::anyhow!(error_message));
        }

        Ok(())
    }

    async fn list_files(&self, rpath: &Path) -> Result<types::ListFileResponse, anyhow::Error> {
        let file_url = format!(
            "{}?path={}",
            &utils::OpsmlPaths::ListFile.as_str(),
            rpath.to_str().unwrap()
        );
        let response = utils::make_get_request(&file_url).await?;
        let files = response.json::<types::ListFileResponse>().await?;
        Ok(files)
    }

    /// Downloads files associated with a model
    ///
    /// # Arguments
    ///
    /// * `rpath` - Remote path to file
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    async fn download_files(&self, rpath: &Path) -> Result<(), anyhow::Error> {
        let rpath_files = self.list_files(rpath).await?;

        println!("Downloading files: {}", rpath_files.files.join(", "));
        // iterate over each file and download
        for file in rpath_files.files.iter() {
            let base_path = rpath;

            // check if rpath is a directory
            let lpath = if rpath.extension().is_none() {
                // if rpath is a directory, append filename to rpath
                let path_to_file = Path::new(file)
                    .strip_prefix(base_path)
                    .with_context(|| "Failed to create file path")?;
                Path::new(self.write_dir).join(path_to_file)
            } else {
                Path::new(self.write_dir).join(
                    Path::new(file)
                        .file_name()
                        .with_context(|| "Failed to create file path")?,
                )
            };

            self.create_dir_path(&lpath)?;
            self.download_file(&lpath, file).await?;
        }

        Ok(())
    }

    /// Downloads a model file
    /// Will also download any associated preprocessor files
    /// Preprocessors can be tokenizer, feature extractor, or preprocessor
    async fn download_model(&self) -> Result<(), anyhow::Error> {
        let download_onnx = !(self.no_onnx); //if no_onnx is true, download_onnx is false
        let model_metadata = self.get_metadata().await?;

        // Get preprocessor
        let preprocessor_rpath = self.get_preprocessor_uri(&model_metadata);

        if preprocessor_rpath.is_some() {
            let preprocessor_rpath = preprocessor_rpath.unwrap();
            self.download_files(&preprocessor_rpath).await?;
        }

        let model_rpath = self.get_model_uri(download_onnx, &model_metadata);

        println!("Downloading model metadata: {:?}", &model_rpath);

        // Get model
        self.download_files(&model_rpath).await?;

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
        quantize: &false,
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
    quantize: &bool,
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
        quantize,
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
    async fn test_list_files() {
        let mut download_server = mockito::Server::new();
        let url = download_server.url();
        env::set_var("OPSML_TRACKING_URI", url);

        let rpath = "./src/api/test_utils/trained_model";

        // directory to write to
        let new_dir = format!("./src/api/test_utils/{}", Uuid::new_v4());

        // get files
        let files_path = "./src/api/test_utils/list_files.json";
        let files = fs::read_to_string(files_path).expect("Unable to read file");
        let list_files: types::ListFileResponse =
            serde_json::from_str(&fs::read_to_string(files_path).expect("Unable to read file"))
                .unwrap();

        // mock list files
        let preprocessor_path = format!("/opsml/files/list?path={}", rpath);
        let mock_list_files = download_server
            .mock("GET", preprocessor_path.as_str())
            .with_status(201)
            .with_body(&files)
            .create();

        let downloader = ModelDownloader {
            name: Some("name"),
            version: Some("version"),
            uid: None,
            write_dir: &new_dir,
            ignore_release_candidates: &false,
            onnx: &true,
            no_onnx: &false,
            quantize: &false,
        };

        let file_response = downloader.list_files(Path::new(rpath)).await.unwrap();
        mock_list_files.assert();

        // assert structs are the same
        assert_json_eq!(list_files, file_response);
    }

    #[tokio::test]
    async fn test_download_model() {
        let mut download_server = mockito::Server::new();
        let url = download_server.url();

        env::set_var("OPSML_TRACKING_URI", url);

        //paths
        let model_path = "./src/api/test_utils/trained_model/sklearn_pipeline-v1-0-0.onnx";
        let preprocessor_path = "./src/api/test_utils/trained_model/tokenizer.json";

        // get metadata
        let metadata_path = "./src/api/test_utils/metadata_onnx.json";
        let metadata = fs::read_to_string(metadata_path).expect("Unable to read file");
        let mock_metadata: types::ModelMetadata =
            serde_json::from_str(&fs::read_to_string(metadata_path).expect("Unable to read file"))
                .unwrap();

        // get files
        let files_path = "./src/api/test_utils/list_files.json";
        let files = fs::read_to_string(files_path).expect("Unable to read file");
        let _: types::ListFileResponse =
            serde_json::from_str(&fs::read_to_string(files_path).expect("Unable to read file"))
                .unwrap();

        // directory to write to
        let new_dir = format!("./src/api/test_utils/{}", Uuid::new_v4());

        // mock metadata
        let mock_metadata_path = download_server
            .mock("POST", "/opsml/models/metadata")
            .with_status(201)
            .with_body(&metadata)
            .create();

        // mock list files
        let _ = download_server
            .mock("GET", "/opsml/files/list")
            .with_status(201)
            .with_body(&files)
            .create();

        // mock model
        let get_path = format!("/opsml/files/download?path={}", model_path);
        let mock_preprocessor_path = download_server
            .mock("GET", get_path.as_str())
            .with_status(201)
            .with_body(&metadata)
            .create();

        // mock preprocessor
        let get_path = format!("/opsml/files/download?path={}", preprocessor_path);
        let mock_model_path = download_server
            .mock("GET", get_path.as_str())
            .with_status(201)
            .with_body(&metadata)
            .create();

        let downloader = ModelDownloader {
            name: Some("name"),
            version: Some("version"),
            uid: None,
            write_dir: &new_dir,
            ignore_release_candidates: &false,
            onnx: &true,
            no_onnx: &false,
            quantize: &false,
        };

        let model_metadata = downloader.get_metadata().await.unwrap();
        mock_metadata_path.assert();

        // assert structs are the same
        assert_json_eq!(mock_metadata, model_metadata);
        downloader.download_model().await.unwrap();

        mock_model_path.assert();
        mock_preprocessor_path.assert();

        // clean up
        //fs::remove_dir_all(&new_dir).unwrap();
    }
}
