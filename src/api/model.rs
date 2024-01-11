/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use crate::api::route_helper::RouteHelper;
use crate::api::types;
use crate::api::utils;
use anyhow::{Context, Result};
use serde_json;
use std::path::PathBuf;
use std::{fs, path::Path};
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
    pub quantize: &'a bool,
}

impl ModelDownloader<'_> {
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

        let response = RouteHelper::make_post_request(
            &utils::OpsmlPaths::MetadataDownload.as_str(),
            &model_metadata_request,
        )
        .await?;

        let loaded_response = RouteHelper::load_stream_response(response).await?;
        let model_metadata: types::ModelMetadata = serde_json::from_str(&loaded_response)
            .with_context(|| "Failed to parse model Metadata")?;

        // create save path for metadata
        utils::create_dir_path(&save_path)?;
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

    /// Downloads files associated with a model
    ///
    /// # Arguments
    ///
    /// * `rpath` - Remote path to file
    ///
    /// # Returns
    /// * `Result<(), String>` - Result of file download
    async fn download_files(&self, rpath: &Path) -> Result<(), anyhow::Error> {
        let rpath_files = RouteHelper::list_files(rpath).await?;

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

            utils::create_dir_path(&lpath)?;
            RouteHelper::download_file(&lpath, file).await?;
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
    async fn test_download_model() {
        let mut download_server = mockito::Server::new();
        let url = download_server.url();

        env::set_var("OPSML_TRACKING_URI", url);

        //paths
        let rpath = "./src/api/test_utils/trained_model";
        let model_path = "./src/api/test_utils/trained_model/sklearn_pipeline-v1-0-0.onnx";
        let tokenizer_path = "./src/api/test_utils/trained_model/tokenizer.json";

        // get metadata
        let metadata_path = "./src/api/test_utils/metadata_onnx.json";
        let metadata = fs::read_to_string(metadata_path).expect("Unable to read file");
        let mock_metadata: types::ModelMetadata =
            serde_json::from_str(&fs::read_to_string(metadata_path).expect("Unable to read file"))
                .unwrap();

        // get files
        let files_path = "./src/api/test_utils/list_files.json";
        let files = fs::read_to_string(files_path).expect("Unable to read file");

        // directory to write to
        let new_dir = format!("./src/api/test_utils/{}", Uuid::new_v4());

        // mock metadata
        let mock_metadata_path = download_server
            .mock("POST", "/opsml/models/metadata")
            .with_status(201)
            .with_body(&metadata)
            .create();

        // mock list files
        let artifact_path = format!("/opsml/files/list?path={}", rpath);
        let mock_list_path = download_server
            .mock("GET", artifact_path.as_str())
            .with_status(201)
            .with_body(&files)
            .create();

        // mock model
        let get_path = format!("/opsml/files/download?path={}", model_path);
        let mock_model_path = download_server
            .mock("GET", get_path.as_str())
            .with_status(201)
            .with_body(&metadata)
            .create();

        // mock preprocessor
        let get_path = format!("/opsml/files/download?path={}", tokenizer_path);
        let mock_tokenizer_path = download_server
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

        mock_list_path.assert();
        mock_model_path.assert();
        mock_tokenizer_path.assert();

        // clean up
        fs::remove_dir_all(&new_dir).unwrap();
    }
}
