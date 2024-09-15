#![feature(duration_constructors)]

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use commands::root::RootCommand;
use config::Config;

mod app;
mod cache;
mod cache_codec;
mod commands;
mod config;
mod fuzzy_matcher;
mod git_provider;
mod projects_list;

#[derive(Parser)]
#[command(author, version, about, long_about = Some("Navigate git projects at the speed of thought"))]
struct Command {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg()]
    search: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Hello {},
}

const DEFAULT_CONFIG_PATH: &str = ".config/gitnow/gitnow.toml";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

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
        Some(_) => todo!(),
        None => {
            RootCommand::new(app).execute(cli.search.as_ref()).await?;
        }
    }

    Ok(())
}
