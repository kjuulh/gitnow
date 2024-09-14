use crate::{ app::App, projects_list::ProjectsListApp};

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

        for repo in &repositories {
            tracing::info!("repo: {}", repo.to_rel_path().display());
        }

        tracing::info!("amount of repos fetched {}", repositories.len());

        Ok(())
    }
}
