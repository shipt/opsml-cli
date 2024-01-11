/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use clap::Args;

#[derive(Args)]
pub struct ListCards {
    /// Name of the registry (data, model, run, etc)
    #[arg(long = "registry")]
    pub registry: String,

    /// Name given to a card
    #[arg(long = "name")]
    pub name: Option<String>,

    /// Team name
    #[arg(long = "team")]
    pub team: Option<String>,

    /// Card version
    #[arg(long = "version")]
    pub version: Option<String>,

    /// Card uid
    #[arg(long = "uid")]
    pub uid: Option<String>,

    /// Card limit
    #[arg(long = "limit")]
    pub limit: Option<i16>,

    /// Tag name
    #[arg(long = "tag_name", use_value_delimiter = true, value_delimiter = ',')]
    pub tag_name: Option<Vec<String>>,

    /// Tag values
    #[arg(long = "tag_value", use_value_delimiter = true, value_delimiter = ',')]
    pub tag_value: Option<Vec<String>>,

    /// max date
    #[arg(long = "max_date")]
    pub max_date: Option<String>,

    /// ignore release candidate
    #[arg(long = "ignore_release_candidate", default_value = "false")]
    pub ignore_release_candidates: bool,
}

#[derive(Args)]
pub struct ModelMetadataArgs {
    /// Name given to card
    #[arg(long = "name")]
    pub name: Option<String>,

    /// Card version
    #[arg(long = "version")]
    pub version: Option<String>,

    /// Card uid
    #[arg(long = "uid")]
    pub uid: Option<String>,

    /// Write directory
    #[arg(long = "write-dir", default_value = ".models")]
    pub write_dir: String,

    /// ignore release candidate
    #[arg(long = "ignore_release_candidate", default_value = "false")]
    pub ignore_release_candidates: bool,
}

#[derive(Args)]
pub struct DownloadModelArgs {
    /// Name given to card
    #[arg(long = "name")]
    pub name: Option<String>,

    /// Card version
    #[arg(long = "version")]
    pub version: Option<String>,

    /// Card uid
    #[arg(long = "uid")]
    pub uid: Option<String>,

    /// Write directory
    #[arg(long = "write-dir", default_value = "models")]
    pub write_dir: String,

    /// Boolean indicating whether to download onnx or trained model
    #[arg(long = "onnx", default_value = "false")]
    pub onnx: bool,

    /// Boolean indicating whether to use the quantized version of the model (huggingface only)
    #[arg(long = "quantize", default_value = "false")]
    pub quantize: bool,

    /// Boolean indicating whether to download any preprocessors with the model
    #[arg(long = "preprocessor", default_value = "false")]
    pub preprocessor: bool,

    /// ignore release candidate
    #[arg(long = "ignore_release_candidate", default_value = "false")]
    pub ignore_release_candidates: bool,
}

#[derive(Args)]
pub struct ModelMetricArgs {
    /// Name given to card
    #[arg(long = "name")]
    pub name: Option<String>,

    /// Card version
    #[arg(long = "version")]
    pub version: Option<String>,

    /// Card uid
    #[arg(long = "uid")]
    pub uid: Option<String>,
}

#[derive(Args)]
pub struct CompareMetricArgs {
    /// Metric name
    #[arg(
        long = "metric-name",
        use_value_delimiter = true,
        value_delimiter = ','
    )]
    pub metric_name: Vec<String>,

    /// If lower is better
    #[arg(
        long = "lower-is-better",
        use_value_delimiter = true,
        value_delimiter = ',',
        default_value = "true"
    )]
    pub lower_is_better: Vec<bool>,

    /// Id of new model challenger
    #[arg(long = "challenger-uid")]
    pub challenger_uid: String,

    /// Id of new model challenger
    #[arg(
        long = "champion-uid",
        use_value_delimiter = true,
        value_delimiter = ',',
        default_value = "true"
    )]
    pub champion_uid: Vec<String>,
}

#[derive(Args)]
pub struct LaunchAppArgs {
    /// Whether to use login credentials
    #[arg(long = "login", default_value = "false")]
    pub login: bool,

    /// Default port to use with the opsml server
    #[arg(long = "port", default_value = "8888")]
    pub port: i32,
}
