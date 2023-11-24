use crate::api::types;
use crate::api::utils;
use owo_colors::OwoColorize;
use reqwest;
use tabled::settings::style::Style;
use tabled::{settings::Alignment, Table};

fn parse_metric_response(response: &str) -> String {
    // Parses response and creates a table

    let metrics: types::ListMetricResponse =
        serde_json::from_str(response).expect("Failed to load response to MetricResponse JSON");

    let mut metric_table: Vec<types::MetricTable> = Vec::new();

    for (_, metric_array) in metrics.metrics.iter() {
        for metric in metric_array.iter() {
            let step = if metric.step.is_some() {
                metric.step.as_ref().unwrap().to_string()
            } else {
                "None".to_string()
            };

            let timestamp = if metric.timestamp.is_some() {
                metric.timestamp.as_ref().unwrap().to_string()
            } else {
                "None".to_string()
            };

            metric_table.push(types::MetricTable {
                metric: metric.name.clone(),
                value: metric.value.clone(),
                step,
                timestamp,
            });
        }
    }

    let metric_table = Table::new(metric_table)
        .with(Alignment::center())
        .with(Style::sharp())
        .to_string();

    metric_table
}

fn parse_compare_metric_response(response: &str) -> String {
    // Parses response and creates a table

    let compare_report: types::CompareMetricResponse = serde_json::from_str(response)
        .expect("Failed to load response to CompareMetricResponse JSON");

    let mut builder = tabled::builder::Builder::default();
    builder.set_header(vec![
        "Champion Name",
        "Champion Version",
        "Metric",
        "Champion Value",
        "Challenger Value",
        "Challenger Win",
    ]);

    let battle_reports = compare_report.report;
    for (_, reports) in battle_reports.iter() {
        for report in reports.iter() {
            // champion and challenger should have metrics to render in table
            if report.champion_metric.is_none() && report.challenger_metric.is_none() {
                continue;
            } else {
                let challenger_metric = report.challenger_metric.as_ref().unwrap();
                let champion_metric = report.champion_metric.as_ref().unwrap();
                let mut record = vec![
                    report.champion_name.clone(),
                    report.champion_version.clone(),
                    champion_metric.name.clone(),
                    champion_metric.value.to_string(),
                    challenger_metric.value.to_string(),
                ];

                if report.challenger_win {
                    record.append(&mut vec!["true".green().to_string()]);
                } else {
                    record.append(&mut vec!["false".red().to_string()]);
                };
                // insert values
                builder.push_record(record);
            }
        }
    }

    let mut table = builder.build();
    let compare_metric_table = table
        .with(Alignment::center())
        .with(Style::sharp())
        .to_string();

    compare_metric_table
}

/// List all metrics for a model
///
/// # Arguments
///
/// * `name` - Name of the model
/// * `version` - Version of the model
/// * `uid` - Unique identifier of the model
/// * `url` - URL of the OpsML server
#[tokio::main]
pub async fn get_model_metrics(
    name: Option<&str>,
    version: Option<&str>,
    uid: Option<&str>,
) -> Result<(), reqwest::Error> {
    let model_metric_request = types::CardRequest {
        name: name.map(|s| s.to_string()),
        version: version.map(|s| s.to_string()),
        uid: uid.map(|s| s.to_string()),
    };

    let response =
        utils::make_post_request(&utils::OpsmlPaths::Metric.as_str(), &model_metric_request).await;

    if response.status().is_success() {
        let metric_table = parse_metric_response(&response.text().await?);
        println!("{}", metric_table);
    } else {
        println!("Failed to get metrics for model");
        response.error_for_status_ref()?;
    }

    Ok(())
}

#[tokio::main]
pub async fn compare_model_metrics(
    metric_name: &[String],
    lower_is_better: &[bool],
    challenger_uid: &str,
    champion_uid: &[String],
) -> Result<(), reqwest::Error> {
    // set up repair request
    let compare_metric_request = types::CompareMetricRequest {
        metric_name: metric_name.to_owned(),
        lower_is_better: lower_is_better.to_owned(),
        challenger_uid: challenger_uid.to_string(),
        champion_uid: champion_uid.to_owned(),
    };

    let response = utils::make_post_request(
        &utils::OpsmlPaths::CompareMetric.as_str(),
        &compare_metric_request,
    )
    .await;

    if response.status().is_success() {
        let metric_table = parse_compare_metric_response(&response.text().await?);
        println!("{}", metric_table);
    } else {
        println!("Failed to get metrics for model");
        response.error_for_status_ref()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_metric_response() {
        let mut vec = Vec::new();
        let metric1 = types::Metric {
            name: "mae".to_string(),
            value: 5.into(),
            step: None,
            timestamp: None,
        };
        vec.push(metric1);

        let metric2 = types::Metric {
            name: "mape".to_string(),
            value: 10.0.into(),
            step: None,
            timestamp: None,
        };
        vec.push(metric2);

        let mut metrics = HashMap::new();
        metrics.insert("test".to_string(), vec);

        let mock_response = types::ListMetricResponse { metrics };
        let string_response = serde_json::to_string(&mock_response).unwrap();

        let metric_table = parse_metric_response(&string_response);

        assert_eq!(
            metric_table,
            concat!(
                "┌────────┬───────┬──────┬───────────┐\n",
                "│ metric │ value │ step │ timestamp │\n",
                "├────────┼───────┼──────┼───────────┤\n",
                "│  mae   │   5   │ None │   None    │\n",
                "│  mape  │ 10.0  │ None │   None    │\n",
                "└────────┴───────┴──────┴───────────┘",
            )
        )
    }

    #[test]
    fn test_parse_compare_metric_response() {
        let challenger_metric = types::Metric {
            name: "mae".to_string(),
            value: 5.into(),
            step: None,
            timestamp: None,
        };

        let champion_metric = types::Metric {
            name: "mape".to_string(),
            value: 10.0.into(),
            step: None,
            timestamp: None,
        };

        let champion_metric2 = types::Metric {
            name: "mape".to_string(),
            value: 2.into(),
            step: None,
            timestamp: None,
        };

        let battle_report = types::BattleReport {
            champion_name: "hootie-and-the-blowfish".to_string(),
            champion_version: "1.0.1".to_string(),
            champion_metric: Some(champion_metric),
            challenger_metric: Some(challenger_metric.clone()),
            challenger_win: true,
        };

        let battle_report2 = types::BattleReport {
            champion_name: "hootie-and-the-blowfish".to_string(),
            champion_version: "1.0.2".to_string(),
            champion_metric: Some(champion_metric2),
            challenger_metric: Some(challenger_metric),
            challenger_win: false,
        };

        let mut report = HashMap::new();
        report.insert("test".to_string(), vec![battle_report, battle_report2]);
        let compare_response = types::CompareMetricResponse {
            challenger_name: "hootie-and-the-blowfish".to_string(),
            challenger_version: "1.0.0".to_string(),
            report,
        };
        let string_response = serde_json::to_string(&compare_response).unwrap();

        parse_compare_metric_response(&string_response);
    }
}
