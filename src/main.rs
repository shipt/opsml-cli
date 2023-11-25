use api::command_structs::{
    CompareMetricArgs, DownloadModelArgs, LaunchAppArgs, ListCards, ModelMetadataArgs,
    ModelMetricArgs,
};
use api::download_file::download_model;
use api::download_file::download_model_metadata;
use api::launch_app::launch_app;
use api::list_cards::list_cards;
use api::metrics::{compare_model_metrics, get_model_metrics};
mod api;
use anyhow::{Context, Result};
use clap::command;
use clap::Parser;
use clap::Subcommand;

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

    /// Launch opsml server with uvicorn
    ///
    /// # Example
    ///
    /// opsml-cli launch-uvicorn-app
    LaunchUvicornApp(LaunchAppArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        // subcommand for list cards
        Some(Commands::ListCards(args)) => {
            let response = list_cards(
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
            );

            response.with_context(|| "Failed to list cards")?;

            Ok(())
        }

        // subcommand for downloading model metadata
        Some(Commands::DownloadModelMetadata(args)) => {
            let response = download_model_metadata(
                args.name.clone(),
                args.version.clone(),
                args.uid.clone(),
                &args.write_dir,
                args.ignore_release_candidates,
            );

            response.with_context(|| {
                format!(
                    "Failed to download model metadata for {:?}",
                    args.name.clone()
                )
            })?;

            Ok(())
        }
        // subcommand for downloading a model
        Some(Commands::DownloadModel(args)) => {
            let response = download_model(
                args.name.clone(),
                args.version.clone(),
                args.uid.clone(),
                &args.write_dir.clone(),
                args.no_onnx,
                args.onnx,
                args.ignore_release_candidates,
            );

            response
                .with_context(|| format!("Failed to download model for {:?}", args.name.clone()))?;
            Ok(())
        }
        // subcommand for getting model metrics
        Some(Commands::GetModelMetrics(args)) => {
            let response = get_model_metrics(
                args.name.as_deref(),
                args.version.as_deref(),
                args.uid.as_deref(),
            );

            response
                .with_context(|| format!("Failed to read instrs from"))
                .unwrap();

            Ok(())
        }

        // subcommand for comparing model metrics
        Some(Commands::CompareModelMetrics(args)) => {
            let response = compare_model_metrics(
                &args.metric_name,
                &args.lower_is_better,
                &args.challenger_uid,
                &args.champion_uid,
            );

            response
                .with_context(|| format!("Failed to read instrs from"))
                .unwrap();

            Ok(())
        }

        // subcommand for launching uvicorn app
        Some(Commands::LaunchUvicornApp(args)) => {
            let response = launch_app(args.port, args.login);

            response
                .with_context(|| format!("Failed to read instrs from"))
                .unwrap();

            Ok(())
        }

        None => Ok(()),
    }
}
