use crate::api::types;
use crate::api::utils;
use anyhow::{Context, Result};
use serde_json;
use std::collections::HashMap;
use tabled::settings::style::Style;
use tabled::{settings::Alignment, Table};

/// Checks if registry is valid
///
/// # Arguments
///
/// * `registry` - Registry to check
///
fn validate_registry(registry: &str) -> Result<(), anyhow::Error> {
    // Determines correct  registry to use

    let registries = ["data", "model", "run", "pipeline", "audit", "project"];

    if registries.contains(&registry) {
        registry.to_string();
        Ok(())
    } else {
        Err(anyhow::Error::msg(format!(
            "Invalid registry: {}. Valid registries are: data, model, run, pipeline, audit, project",
            registry.to_string()
        )))
    }
}

/// Parse card list response
///
/// # Arguments
///
/// * `response` - Response from server
///
/// # Returns
///  String - Table of cards
///
fn parse_list_response(response: &str) -> Result<String, anyhow::Error> {
    // Parses response and creates a table

    let cards: types::ListCardResponse = serde_json::from_str(response)
        .with_context(|| format!("Failed to load response to ListCardResponse JSON"))
        .unwrap();

    let mut card_table: Vec<types::CardTable> = Vec::new();

    for card in cards.cards.iter() {
        card_table.push(types::CardTable {
            name: card.name.clone(),
            team: card.team.clone(),
            date: card.date.clone(),
            user_email: card.user_email.clone(),
            version: card.version.clone(),
            uid: card.uid.clone(),
        });
    }

    let list_table = Table::new(card_table)
        .with(Alignment::center())
        .with(Style::sharp())
        .to_string();

    Ok(list_table)
}

/// List cards
///     
/// # Arguments
///
/// * `registry` - Registry to list cards from
/// * `name` - Name of card
/// * `team` - Team name
/// * `version` - Card version
/// * `uid` - Card uid
/// * `limit` - Limit number of cards returned
/// * `url` - OpsML url
/// * `tag_name` - Tag name
/// * `tag_value` - Tag value
/// * `max_date` - Max date
///
#[tokio::main]
#[allow(clippy::too_many_arguments)]
pub async fn list_cards(
    registry: &str,
    name: Option<&str>,
    team: Option<&str>,
    version: Option<&str>,
    uid: Option<&str>,
    limit: Option<i16>,
    tag_name: Option<Vec<String>>,
    tag_value: Option<Vec<String>>,
    max_date: Option<&str>,
    ignore_release_candidates: bool,
) -> Result<(), anyhow::Error> {
    // set full path and table name

    // create empty dict for tags
    let mut tags: HashMap<String, String> = HashMap::new();

    let _valid = validate_registry(registry)?;

    if tag_name.is_some() && tag_value.is_some() {
        tags = tag_name
            .unwrap()
            .iter()
            .zip(tag_value.unwrap().iter())
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
    }

    let list_table_request = types::ListTableRequest {
        registry_type: registry.to_string(),
        name: name.map(|s| s.to_string()),
        team: team.map(|s| s.to_string()),
        version: version.map(|s| s.to_string()),
        limit,
        uid: uid.map(|s| s.to_string()),
        tags: Some(tags),
        max_date: max_date.map(|s| s.to_string()),
        ignore_release_candidates,
    };

    let response =
        utils::make_post_request(&utils::OpsmlPaths::ListCard.as_str(), &list_table_request)
            .await
            .unwrap();

    if response.status().is_success() {
        let card_table = parse_list_response(&response.text().await.unwrap());
        println!("{}", card_table?);
        Ok(())
    } else {
        Err(anyhow::Error::msg(format!(
            "Failed to make call to list cards: {}",
            response.text().await.unwrap()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_registry() {
        let v = vec!["data", "model", "run", "audit"];

        for name in &v {
            validate_registry(name).unwrap();
        }
    }

    #[test]
    fn test_parse_response() {
        let mut vec = Vec::new();
        let card = types::Card {
            name: "test".to_string(),
            team: "test".to_string(),
            date: "test".to_string(),
            user_email: "fake_email".to_string(),
            version: "1.0.0".to_string(),
            uid: "uid".to_string(),
            tags: HashMap::new(),
        };
        vec.push(card);

        let mock_response = types::ListCardResponse { cards: vec };
        let string_response = serde_json::to_string(&mock_response).unwrap();

        let card_table = parse_list_response(&string_response);
        assert_eq!(
            card_table.unwrap(),
            concat!(
                "┌──────┬──────┬──────┬────────────┬─────────┬─────┐\n",
                "│ name │ team │ date │ user_email │ version │ uid │\n",
                "├──────┼──────┼──────┼────────────┼─────────┼─────┤\n",
                "│ test │ test │ test │ fake_email │  1.0.0  │ uid │\n",
                "└──────┴──────┴──────┴────────────┴─────────┴─────┘",
            )
        );
    }
}
