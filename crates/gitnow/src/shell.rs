use std::path::Path;

use anyhow::Context;

use crate::{app::App, git_provider::Repository};

pub struct Shell {
    app: &'static App,
}

impl Shell {
    pub fn new(app: &'static App) -> Self {
        Self { app }
    }

    pub async fn spawn_shell(&self, repository: &Repository) -> anyhow::Result<()> {
        let project_path = self
            .app
            .config
            .settings
            .projects
            .directory
            .join(repository.to_rel_path());

        self.spawn_shell_at(&project_path).await
    }

    pub async fn spawn_shell_at(&self, path: &Path) -> anyhow::Result<()> {
        if !path.exists() {
            anyhow::bail!(
                "project path: {} does not exists, it is either a file, or hasn't been cloned",
                path.display()
            );
        }

        let shell = std::env::var("SHELL")
            .context("failed to find SHELL variable, required for spawning embedded shells")?;

        let mut shell_cmd = tokio::process::Command::new(shell);
        shell_cmd.current_dir(path);

        let mut process = shell_cmd.spawn().context("failed to spawn child session")?;

        let status = process.wait().await?;

        if !status.success() {
            tracing::warn!(
                "child session returned non-zero, or missing return code: {}",
                status.code().unwrap_or_default()
            );
            anyhow::bail!(
                "child shell session failed with exit: {}",
                status.code().unwrap_or(-1)
            );
        } else {
            tracing::debug!("child session returned 0 exit code");
        }

        Ok(())
    }
}

pub trait ShellApp {
    fn shell(&self) -> Shell;
}

impl ShellApp for &'static App {
    fn shell(&self) -> Shell {
        Shell::new(self)
    }
}
