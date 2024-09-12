use anyhow::Context;
use clap::{Parser, Subcommand};
use commands::root::RootCommand;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let app = app::App::new_static();

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

mod config {
    use std::path::Path;

    use anyhow::Context;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct Config {
        #[serde(default)]
        pub providers: Providers,
    }

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
    pub struct Providers {
        #[serde(default)]
        pub github: Vec<GitHub>,
        #[serde(default)]
        pub gitea: Vec<Gitea>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct GitHub {
        #[serde(default)]
        pub users: Vec<GitHubUser>,
        #[serde(default)]
        pub organisations: Vec<GitHubOrganisation>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct GitHubUser(String);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct GitHubOrganisation(String);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct Gitea {
        #[serde(default)]
        pub users: Vec<GiteaUser>,
        #[serde(default)]
        pub organisations: Vec<GiteaOrganisation>,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct GiteaUser(String);

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct GiteaOrganisation(String);

    impl Config {
        pub async fn from_file(file_path: &Path) -> anyhow::Result<Config> {
            let file_content = tokio::fs::read_to_string(file_path).await?;

            Self::from_string(&file_content)
        }

        pub fn from_string(content: &str) -> anyhow::Result<Config> {
            toml::from_str(content).context("failed to deserialize config file")
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_can_parse_config() -> anyhow::Result<()> {
            let content = r#"
              [[providers.github]]  
              users = ["kjuulh"]
              organisations = ["lunarway"]

              [[providers.github]]  
              users = ["other"]
              organisations = ["org"]

              [[providers.gitea]]  
              users = ["kjuulh"]
              organisations = ["lunarway"]

              [[providers.gitea]]  
              users = ["other"]
              organisations = ["org"]

              [[providers.gitea]]  
            "#;

            let config = Config::from_string(content)?;

            assert_eq!(
                Config {
                    providers: Providers {
                        github: vec![
                            GitHub {
                                users: vec![GitHubUser("kjuulh".into())],
                                organisations: vec![GitHubOrganisation("lunarway".into())]
                            },
                            GitHub {
                                users: vec![GitHubUser("other".into())],
                                organisations: vec![GitHubOrganisation("org".into())]
                            }
                        ],
                        gitea: vec![
                            Gitea {
                                users: vec![GiteaUser("kjuulh".into())],
                                organisations: vec![GiteaOrganisation("lunarway".into())]
                            },
                            Gitea {
                                users: vec![GiteaUser("other".into())],
                                organisations: vec![GiteaOrganisation("org".into())]
                            },
                            Gitea {
                                users: vec![],
                                organisations: vec![]
                            },
                        ]
                    }
                },
                config
            );

            Ok(())
        }

        #[test]
        fn test_can_parse_empty_config() -> anyhow::Result<()> {
            let content = r#"
                # empty file
            "#;

            let config = Config::from_string(content)?;

            assert_eq!(
                Config {
                    providers: Providers {
                        github: vec![],
                        gitea: vec![]
                    }
                },
                config
            );

            Ok(())
        }
    }
}

mod git_provider {
    use async_trait::async_trait;

    pub struct Repository {}

    #[async_trait]
    pub trait GitProvider {
        async fn list_repositories(&self) -> anyhow::Result<Vec<Repository>>;
    }
}

mod app {
    #[derive(Debug)]
    pub struct App {}

    impl App {
        pub fn new_static() -> &'static App {
            Box::leak(Box::new(App {}))
        }
    }
}

mod commands {
    pub mod root {
        use crate::app::App;

        #[derive(Debug, Clone)]
        pub struct RootCommand {
            app: &'static App,
        }

        impl RootCommand {
            pub fn new(app: &'static App) -> Self {
                Self { app }
            }

            #[tracing::instrument]
            pub async fn execute(&mut self) -> anyhow::Result<()> {
                tracing::debug!("executing");

                Ok(())
            }
        }
    }
}
