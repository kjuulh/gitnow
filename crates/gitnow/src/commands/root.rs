use crate::{
    app::App,
    git_provider::{
        gitea::GiteaProviderApp, github::GitHubProviderApp, GitProvider, VecRepositoryExt,
    },
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

        //let github_provider = self.app.github_provider();
        let gitea_provider = self.app.gitea_provider();

        let mut repositories = Vec::new();
        for gitea in self.app.config.providers.gitea.iter() {
            if let Some(user) = &gitea.current_user {
                let mut repos = gitea_provider
                    .list_repositories_for_current_user(
                        user,
                        &gitea.url,
                        gitea.access_token.as_ref(),
                    )
                    .await?;

                repositories.append(&mut repos);
            }

            for gitea_user in gitea.users.iter() {
                let mut repos = gitea_provider
                    .list_repositories_for_user(
                        gitea_user.into(),
                        &gitea.url,
                        gitea.access_token.as_ref(),
                    )
                    .await?;

                repositories.append(&mut repos);
            }
        }

        repositories.collect_unique();

        for repo in &repositories {
            tracing::info!("repo: {}", repo.to_rel_path().display());
        }

        tracing::info!("amount of repos fetched {}", repositories.len());

        Ok(())
    }
}
