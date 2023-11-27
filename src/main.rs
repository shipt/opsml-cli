/// Copyright (c) Shipt, Inc.
/// This source code is licensed under the MIT license found in the
/// LICENSE file in the root directory of this source tree.
use api::command_structs::{
    CompareMetricArgs, DownloadModelArgs, ListCards, ModelMetadataArgs, ModelMetricArgs,
};
use api::download_file::download_model;
use api::download_file::download_model_metadata;
use api::list_cards::list_cards;
use api::metrics::{compare_model_metrics, get_model_metrics};
mod api;
use anyhow::{Context, Result};
use clap::command;
use clap::Parser;
use clap::Subcommand;
use owo_colors::OwoColorize;

#[derive(Parser)]
#[command(about = "CLI tool for Interacting with an Opsml server")]

struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Lists cards from a registry
    ///
    /// # Example
    ///
    /// opsml-cli list-cards --registry data
    ListCards(ListCards),
    /// Download model metadata from the model registry
    ///
    /// # Example
    ///
    /// opsml-cli download-model-metadata --name model_name --version 1.0.0
    DownloadModelMetadata(ModelMetadataArgs),
    /// Download a model and its metadata from the model registry
    ///
    /// # Example
    ///
    /// opsml-cli download-model --name model_name --version 1.0.0
    /// opsml-cli download-model --name model_name --version 1.0.0 --no-onnx
    DownloadModel(DownloadModelArgs),
    /// Retrieve model metrics
    ///
    /// # Example
    ///
    /// opsml-cli get-model-metrics --name model_name --version 1.0.0
    GetModelMetrics(ModelMetricArgs),
    /// Compare model metrics
    ///
    /// # Example
    ///
    /// opsml-cli compare-model-metrics
    CompareModelMetrics(CompareMetricArgs),

    ///  Show opsml-cli version
    ///
    /// # Example
    ///
    /// opsml-cli version
    Version,

    ///  Show opsml-cli info
    ///
    /// # Example
    ///
    /// opsml-cli info
    Info,
}

const LOGO_TEXT: &str = "
 ██████  ██████  ███████ ███    ███ ██             ██████ ██      ██ 
██    ██ ██   ██ ██      ████  ████ ██            ██      ██      ██ 
██    ██ ██████  ███████ ██ ████ ██ ██      █████ ██      ██      ██ 
██    ██ ██           ██ ██  ██  ██ ██            ██      ██      ██ 
 ██████  ██      ███████ ██      ██ ███████        ██████ ███████ ██ 
";

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        // subcommand for list cards
        Some(Commands::ListCards(args)) => {
            list_cards(
                args.registry.as_str(),
                args.name.as_deref(),
                args.team.as_deref(),
                args.version.as_deref(),
                args.uid.as_deref(),
                args.limit,
                args.tag_name.clone(),
                args.tag_value.clone(),
                args.max_date.as_deref(),
                args.ignore_release_candidates,
            )
            .with_context(|| format!("{}", "Failed to list cards".bold().red()))?;

            Ok(())
        }

        // subcommand for downloading model metadata
        Some(Commands::DownloadModelMetadata(args)) => {
            download_model_metadata(
                args.name.clone(),
                args.version.clone(),
                args.uid.clone(),
                &args.write_dir,
                args.ignore_release_candidates,
            )
            .with_context(|| {
                format!(
                    "Failed to download model metadata for {:?}",
                    args.name.clone().bold().red()
                )
            })?;

            Ok(())
        }
        // subcommand for downloading a model
        Some(Commands::DownloadModel(args)) => {
            download_model(
                args.name.clone(),
                args.version.clone(),
                args.uid.clone(),
                &args.write_dir.clone(),
                args.no_onnx,
                args.onnx,
                args.ignore_release_candidates,
            )
            .with_context(|| {
                format!(
                    "Failed to download model for {:?}",
                    args.name.clone().bold().red()
                )
            })?;
            Ok(())
        }
        // subcommand for getting model metrics
        Some(Commands::GetModelMetrics(args)) => {
            get_model_metrics(
                args.name.as_deref(),
                args.version.as_deref(),
                args.uid.as_deref(),
            )
            .with_context(|| {
                format!(
                    "Failed to get model metrics for {:?}",
                    args.name.clone().bold().red()
                )
            })?;

            Ok(())
        }

        // subcommand for comparing model metrics
        Some(Commands::CompareModelMetrics(args)) => {
            compare_model_metrics(
                &args.metric_name,
                &args.lower_is_better,
                &args.challenger_uid,
                &args.champion_uid,
            )
            .with_context(|| {
                format!(
                    "Failed to compare model metrics for {:?}",
                    &args.champion_uid.bold().red()
                )
            })?;

            Ok(())
        }

        // subcommand for listing opsml-cli version
        Some(Commands::Version) => {
            println!(
                "opsml-cli version {}",
                env!("CARGO_PKG_VERSION").bold().green()
            );
            Ok(())
        }

        // subcommand for listing opsml-cli info
        Some(Commands::Info) => {
            println!(
                "\n{}\nopsml-cli version {}\n2023 Shipt, Inc.\n",
                LOGO_TEXT.green(),
                env!("CARGO_PKG_VERSION").bold().purple(),
            );

            Ok(())
        }

        None => Ok(()),
    }
}
