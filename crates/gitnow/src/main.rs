use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use commands::{
    clone::CloneCommand, project::ProjectCommand, root::RootCommand, shell::Shell, update::Update,
    worktree::WorktreeCommand,
};
use config::Config;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod app;
mod cache;
mod cache_codec;
pub mod chooser;
mod commands;
mod components;
mod config;
mod custom_command;
mod fuzzy_matcher;
mod git_clone;
mod git_provider;
mod interactive;
mod projects_list;
mod shell;
mod template_command;
mod worktree;

#[derive(Parser)]
#[command(author, version, about, long_about = Some("Navigate git projects at the speed of thought"))]
struct Command {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to config file (default: ~/.config/gitnow/gitnow.toml, or $GITNOW_CONFIG)
    #[arg(long = "config", short = 'c', global = true)]
    config: Option<PathBuf>,

    #[arg()]
    search: Option<String>,

    #[arg(long = "no-cache", default_value = "false")]
    no_cache: bool,

    #[arg(long = "no-clone", default_value = "false")]
    no_clone: bool,

    #[arg(long = "no-shell", default_value = "false")]
    no_shell: bool,

    /// Path to a chooser file; if set, the selected directory path is written
    /// to this file instead of spawning a shell or printing to stdout.
    /// Can also be set via the GITNOW_CHOOSER_FILE environment variable.
    #[arg(long = "chooser-file", global = true, env = "GITNOW_CHOOSER_FILE")]
    chooser_file: Option<PathBuf>,

    #[arg(long = "force-refresh", default_value = "false")]
    force_refresh: bool,

    #[arg(long = "force-cache-update", default_value = "false")]
    force_cache_update: bool,
}

#[derive(Subcommand)]
enum Commands {
    Init(Shell),
    Update(Update),
    Clone(CloneCommand),
    Worktree(WorktreeCommand),
    /// Manage scratch-pad projects with multiple repositories
    Project(ProjectCommand),
}

const DEFAULT_CONFIG_PATH: &str = ".config/gitnow/gitnow.toml";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::ERROR.into())
                .from_env_lossy(),
        )
        .init();

    let cli = Command::parse();
    tracing::debug!("Starting cli");

    let config_path = if let Some(path) = &cli.config {
        path.clone()
    } else {
        let home =
            std::env::var("HOME").context("HOME was not found, are you using a proper shell?")?;
        let default_config_path = PathBuf::from(home).join(DEFAULT_CONFIG_PATH);
        std::env::var("GITNOW_CONFIG")
            .map(PathBuf::from)
            .unwrap_or(default_config_path)
    };

    let config = Config::from_file(&config_path).await?;

    let app = app::App::new_static(config).await?;

    // When a chooser file is provided, it implies --no-shell behaviour:
    // the selected path is written to the file and no interactive shell is
    // spawned.  The calling shell wrapper is responsible for reading the
    // file and changing directory.
    let chooser = cli
        .chooser_file
        .map(chooser::Chooser::new)
        .unwrap_or_default();
    let no_shell = cli.no_shell || chooser.is_active();

    match cli.command {
        Some(cmd) => match cmd {
            Commands::Init(mut shell) => {
                shell.execute().await?;
            }
            Commands::Update(mut update) => {
                update.execute(app).await?;
            }
            Commands::Clone(mut clone) => {
                clone.execute(app).await?;
            }
            Commands::Worktree(mut wt) => {
                wt.execute(app, &chooser).await?;
            }
            Commands::Project(mut project) => {
                project.execute(app, &chooser).await?;
            }
        },
        None => {
            RootCommand::new(app)
                .execute(
                    cli.search.as_ref(),
                    !cli.no_cache,
                    !cli.no_clone,
                    !no_shell,
                    cli.force_refresh,
                    cli.force_cache_update,
                    &chooser,
                )
                .await?;
        }
    }

    Ok(())
}
