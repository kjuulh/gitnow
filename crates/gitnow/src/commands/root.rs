use crate::{
    app::App,
    git_provider::{
        gitea::GiteaProviderApp, github::GitHubProviderApp, GitProvider, VecRepositoryExt,
    },
    projects_list::ProjectsListApp,
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

        let repositories = self.app.projects_list().get_projects().await?;

        //let github_provider = self.app.github_provider();
        for repo in &repositories {
            tracing::info!("repo: {}", repo.to_rel_path().display());
        }

        tracing::info!("amount of repos fetched {}", repositories.len());

        Ok(())
    }
}
