use crate::{app::App, components::inline_command::InlineCommand, git_provider::Repository};

#[derive(Debug, Clone)]
pub struct GitClone {
    app: &'static App,
}

impl GitClone {
    pub fn new(app: &'static App) -> Self {
        Self { app }
    }

    pub async fn clone_repo(
        &self,
        repository: &Repository,
        force_refresh: bool,
    ) -> anyhow::Result<()> {
        let project_path = self
            .app
            .config
            .settings
            .projects
            .directory
            .join(repository.to_rel_path());

        if force_refresh {
            tokio::fs::remove_dir_all(&project_path).await?;
        }

        if project_path.exists() {
            tracing::info!(
                "project: {} already exists, skipping clone",
                repository.to_rel_path().display()
            );
            return Ok(());
        }

        tracing::info!(
            "cloning: {} into {}",
            repository.ssh_url.as_str(),
            &project_path.display().to_string(),
        );

        let mut cmd = tokio::process::Command::new("git");
        cmd.args([
            "clone",
            repository.ssh_url.as_str(),
            &project_path.display().to_string(),
        ]);

        let output = cmd.output().await?;
        match output.status.success() {
            true => tracing::debug!(
                "cloned {} into {}",
                repository.ssh_url.as_str(),
                &project_path.display().to_string(),
            ),
            false => {
                let stdout = std::str::from_utf8(&output.stdout).unwrap_or_default();
                let stderr = std::str::from_utf8(&output.stderr).unwrap_or_default();
                tracing::error!(
                    "failed to clone {} into {}, with output: {}, err: {}",
                    repository.ssh_url.as_str(),
                    &project_path.display().to_string(),
                    stdout,
                    stderr
                )
            }
        }

        Ok(())
    }
}

pub trait GitCloneApp {
    fn git_clone(&self) -> GitClone;
}

impl GitCloneApp for &'static App {
    fn git_clone(&self) -> GitClone {
        GitClone::new(self)
    }
}
