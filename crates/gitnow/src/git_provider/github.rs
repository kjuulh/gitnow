use async_trait::async_trait;

use crate::app::App;

use super::GitProvider;

pub struct GitHubProvider {
    app: &'static App,
}

impl GitHubProvider {
    pub fn new(app: &'static App) -> GitHubProvider {
        GitHubProvider { app }
    }
}

#[async_trait]
impl GitProvider for GitHubProvider {
    async fn list_repositories_for_user(
        &self,
        user: &str,
    ) -> anyhow::Result<Vec<super::Repository>> {
        todo!()
    }

    async fn list_repositories_for_organisation(
        &self,
        organisation: &str,
    ) -> anyhow::Result<Vec<super::Repository>> {
        todo!()
    }
}

pub trait GitHubProviderApp {
    fn github_provider(&self) -> GitHubProvider;
}

impl GitHubProviderApp for &'static App {
    fn github_provider(&self) -> GitHubProvider {
        GitHubProvider::new(self)
    }
}
