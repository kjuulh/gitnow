use crate::{app::App, cache::CacheApp, projects_list::ProjectsListApp};

#[derive(clap::Parser)]
pub struct Update {}

impl Update {
    pub async fn execute(&mut self, app: &'static App) -> anyhow::Result<()> {
        let repositories = app.projects_list().get_projects().await?;

        app.cache().update(&repositories).await?;

        Ok(())
    }
}
