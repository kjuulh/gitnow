use crate::{
    app::App,
    git_provider::{gitea::GiteaProviderApp, Repository, VecRepositoryExt},
};

pub struct ProjectsList {
    app: &'static App,
}

impl ProjectsList {
    pub fn new(app: &'static App) -> Self {
        Self { app }
    }

    pub async fn get_projects(&self) -> anyhow::Result<Vec<Repository>> {
        let mut repositories = Vec::new();

        repositories.extend(self.get_gitea_projects().await?);

        repositories.collect_unique();

        Ok(repositories)
    }

    async fn get_gitea_projects(&self) -> anyhow::Result<Vec<Repository>> {
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

            for gitea_org in gitea.organisations.iter() {
                let mut repos = gitea_provider
                    .list_repositories_for_organisation(
                        gitea_org.into(),
                        &gitea.url,
                        gitea.access_token.as_ref(),
                    )
                    .await?;

                repositories.append(&mut repos);
            }
        }

        Ok(repositories)
    }
}

pub trait ProjectsListApp {
    fn projects_list(&self) -> ProjectsList;
}

impl ProjectsListApp for &'static App {
    fn projects_list(&self) -> ProjectsList {
        ProjectsList::new(self)
    }
}
