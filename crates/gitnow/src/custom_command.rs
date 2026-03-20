use std::path::Path;

use crate::{app::App, config::CommandList};

pub struct CustomCommand {
    post_clone: Option<CommandList>,
    post_update: Option<CommandList>,
}

impl CustomCommand {
    pub fn new(app: &App) -> Self {
        Self {
            post_clone: app.config.settings.post_clone_command.clone(),
            post_update: app.config.settings.post_update_command.clone(),
        }
    }

    async fn execute_commands(
        commands: &CommandList,
        path: &Path,
        label: &str,
    ) -> anyhow::Result<()> {
        for command in commands.get_commands() {
            let command_parts = command.split(' ').collect::<Vec<_>>();
            let Some((first, rest)) = command_parts.split_first() else {
                return Ok(());
            };

            let mut cmd = tokio::process::Command::new(first);
            cmd.args(rest).current_dir(path);

            eprintln!("running command: {}", command);

            tracing::info!(
                path = path.display().to_string(),
                cmd = command,
                "running custom {} command",
                label
            );
            let output = cmd.output().await?;
            let stdout = std::str::from_utf8(&output.stdout)?;
            tracing::info!(
                stdout = stdout,
                "finished running custom {} command",
                label
            );
        }

        Ok(())
    }

    pub async fn execute_post_clone_command(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(ref cmds) = self.post_clone {
            Self::execute_commands(cmds, path, "post clone").await?;
        }
        Ok(())
    }

    pub async fn execute_post_update_command(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(ref cmds) = self.post_update {
            Self::execute_commands(cmds, path, "post update").await?;
        }
        Ok(())
    }
}

pub trait CustomCommandApp {
    fn custom_command(&self) -> CustomCommand;
}

impl CustomCommandApp for App {
    fn custom_command(&self) -> CustomCommand {
        CustomCommand::new(self)
    }
}
