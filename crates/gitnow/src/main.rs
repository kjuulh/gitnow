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

mod config;

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
