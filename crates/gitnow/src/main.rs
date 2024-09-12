use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use commands::root::RootCommand;
use config::Config;

mod config;
mod git_provider;

#[derive(Parser)]
#[command(author, version, about, long_about = Some("Navigate git projects at the speed of thought"))]
struct Command {
    #[command(subcommand)]
    command: Option<Commands>,
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
            RootCommand::new(app).execute().await?;
        }
    }

    Ok(())
}

mod app {
    use crate::config::Config;

    #[derive(Debug)]
    pub struct App {
        pub config: Config,
    }

    impl App {
        pub async fn new_static(config: Config) -> anyhow::Result<&'static App> {
            Ok(Box::leak(Box::new(App { config })))
        }
    }
}

mod commands {
    pub mod root {
        use crate::{
            app::App,
            git_provider::{
                gitea::GiteaProviderApp, github::GitHubProviderApp, GitProvider, VecRepositoryExt,
            },
        };

        #[derive(Debug, Clone)]
        pub struct RootCommand {
            app: &'static App,
        }

        impl RootCommand {
            pub fn new(app: &'static App) -> Self {
                Self { app }
            }

            #[tracing::instrument(skip(self))]
            pub async fn execute(&mut self) -> anyhow::Result<()> {
                tracing::debug!("executing");

                //let github_provider = self.app.github_provider();
                let gitea_provider = self.app.gitea_provider();

                let mut repositories = Vec::new();
                for gitea in self.app.config.providers.gitea.iter() {
                    if let Some(user) = &gitea.current_user {
                        let mut repos = gitea_provider
                            .list_repositories_for_current_user(
                                user,
                                &gitea.url,
                                gitea.access_token.as_ref(),
                            )
                            .await?;

                        repositories.append(&mut repos);
                    }

                    for gitea_user in gitea.users.iter() {
                        let mut repos = gitea_provider
                            .list_repositories_for_user(
                                gitea_user.into(),
                                &gitea.url,
                                gitea.access_token.as_ref(),
                            )
                            .await?;

                        repositories.append(&mut repos);
                    }
                }

                repositories.collect_unique();

                for repo in &repositories {
                    tracing::info!("repo: {}", repo.to_rel_path().display());
                }

                tracing::info!("amount of repos fetched {}", repositories.len());

                Ok(())
            }
        }
    }
}
