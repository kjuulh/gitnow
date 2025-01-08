use std::path::Path;

use crate::{app::App, config::Config};

pub struct CustomCommand {
    config: Config,
}

impl CustomCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn execute_post_update_command(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(post_update_command) = &self.config.settings.post_update_command {
            let command_parts = post_update_command.split(' ').collect::<Vec<_>>();
            let Some((first, rest)) = command_parts.split_first() else {
                return Ok(());
            };

            let mut cmd = tokio::process::Command::new(first);
            cmd.args(rest).current_dir(path);

            tracing::info!(
                path = path.display().to_string(),
                cmd = post_update_command,
                "running custom post update command"
            );
            let output = cmd.output().await?;
            let stdout = std::str::from_utf8(&output.stdout)?;
            tracing::info!(
                stdout = stdout,
                "finished running custom post update command"
            );
        }

        Ok(())
    }
}

pub trait CustomCommandApp {
    fn custom_command(&self) -> CustomCommand;
}

impl CustomCommandApp for App {
    fn custom_command(&self) -> CustomCommand {
        CustomCommand::new(self.config.clone())
    }
}
