#![feature(duration_constructors)]

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use commands::{root::RootCommand, shell::Shell};
use config::Config;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod app;
mod cache;
mod cache_codec;
mod commands;
mod components;
mod config;
mod fuzzy_matcher;
mod git_clone;
mod git_provider;
mod interactive;
mod projects_list;
mod shell;

#[derive(Parser)]
#[command(author, version, about, long_about = Some("Navigate git projects at the speed of thought"))]
struct Command {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg()]
    search: Option<String>,

    #[arg(long = "no-cache", default_value = "false")]
    no_cache: bool,

    #[arg(long = "no-clone", default_value = "false")]
    no_clone: bool,

    #[arg(long = "no-shell", default_value = "false")]
    no_shell: bool,

    #[arg(long = "force-refresh", default_value = "false")]
    force_refresh: bool,

    #[arg(long = "force-cache-update", default_value = "false")]
    force_cache_update: bool,
}

#[derive(Subcommand)]
enum Commands {
    Init(Shell),
}

const DEFAULT_CONFIG_PATH: &str = ".config/gitnow/gitnow.toml";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::ERROR.into())
                .from_env_lossy(),
        )
        .init();

    let home =
        std::env::var("HOME").context("HOME was not found, are you using a proper shell?")?;
    let default_config_path = PathBuf::from(home).join(DEFAULT_CONFIG_PATH);
    let config_path = std::env::var("GITNOW_CONFIG")
        .map(PathBuf::from)
        .unwrap_or(default_config_path);

    let config = Config::from_file(&config_path).await?;

    let app = app::App::new_static(config).await?;

    let cli = Command::parse();
    tracing::debug!("Starting cli");

    match cli.command {
        Some(Commands::Init(mut shell)) => shell.execute().await?,
        None => {
            RootCommand::new(app)
                .execute(
                    cli.search.as_ref(),
                    !cli.no_cache,
                    !cli.no_clone,
                    !cli.no_shell,
                    cli.force_refresh,
                    cli.force_cache_update,
                )
                .await?;
        }
    }

    Ok(())
}
