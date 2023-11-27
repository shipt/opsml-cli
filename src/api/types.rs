/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tabled::Tabled;

#[derive(Debug, Serialize)]
pub struct ListTableRequest<'a> {
    pub registry_type: &'a str,
    pub name: Option<&'a str>,
    pub team: Option<&'a str>,
    pub version: Option<&'a str>,
    pub uid: Option<&'a str>,
    pub limit: Option<&'a i16>,
    pub tags: &'a HashMap<String, String>,
    pub max_date: Option<&'a str>,
    pub ignore_release_candidates: &'a bool,
}

#[derive(Debug, Serialize)]
pub struct CardRequest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub uid: Option<String>,
}

#[derive(Serialize)]
pub struct ModelMetadataRequest {
    pub name: Option<String>,
    pub version: Option<String>,
    pub uid: Option<String>,
    pub ignore_release_candidates: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    pub team: String,
    pub date: String,
    pub user_email: String,
    pub version: String,
    pub uid: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMetricResponse {
    pub metrics: HashMap<String, Vec<Metric>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metric {
    pub name: String,
    pub value: Value,
    pub step: Option<Value>,
    pub timestamp: Option<Value>,
}

#[derive(Tabled)]
pub struct MetricTable {
    pub metric: String,
    pub value: Value,
    pub step: String,
    pub timestamp: String,
}

#[derive(Tabled)]
pub struct CompareMetricTable {
    pub champion_name: String,
    pub champion_version: Value,
    pub metric: String,
    pub champion_value: Value,
    pub challenger_value: Value,
    pub challenger_win: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCardResponse {
    pub cards: Vec<Card>,
}

#[derive(Tabled)]
pub struct CardTable {
    pub name: String,
    pub team: String,
    pub date: String,
    pub user_email: String,
    pub version: String,
    pub uid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDataSchema {
    data_type: String,
    input_features: HashMap<String, Value>,
    output_features: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSchema {
    model_data_schema: ModelDataSchema,
    input_data_schema: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_name: String,
    pub model_type: String,
    pub onnx_uri: Option<String>,
    pub onnx_version: Option<String>,
    pub model_uri: String,
    pub model_version: String,
    pub model_team: String,
    pub sample_data: HashMap<String, Value>,
    pub data_schema: DataSchema,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompareMetricRequest {
    pub metric_name: Vec<String>,
    pub lower_is_better: Vec<bool>,
    pub challenger_uid: String,
    pub champion_uid: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BattleReport {
    pub champion_name: String,
    pub champion_version: String,
    pub champion_metric: Option<Metric>,
    pub challenger_metric: Option<Metric>,
    pub challenger_win: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompareMetricResponse {
    pub challenger_name: String,
    pub challenger_version: String,
    pub report: HashMap<String, Vec<BattleReport>>,
}
