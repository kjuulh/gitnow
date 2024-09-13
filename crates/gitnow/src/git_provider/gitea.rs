use anyhow::Context;
use gitea_rs::apis::configuration::{ApiKey, Configuration};
use url::Url;

use crate::{app::App, config::GiteaAccessToken};

#[derive(Debug)]
pub struct GiteaProvider {
    app: &'static App,
}

impl GiteaProvider {
    pub fn new(app: &'static App) -> GiteaProvider {
        GiteaProvider { app }
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_repositories_for_current_user(
        &self,
        user: &str,
        api: &str,
        access_token: Option<&GiteaAccessToken>,
    ) -> anyhow::Result<Vec<super::Repository>> {
        tracing::debug!("fetching gitea repositories for user");

        let config = self.get_config(api, access_token)?;

        let mut repositories = Vec::new();
        let mut page = 1;
        loop {
            let mut repos = self
                .list_repositories_for_current_user_with_page(user, &config, page)
                .await?;

            if repos.is_empty() {
                break;
            }

            repositories.append(&mut repos);
            page += 1;
        }

        let provider = &Self::get_domain(api)?;

        Ok(repositories
            .into_iter()
            .map(|repo| super::Repository {
                provider: provider.into(),
                owner: repo
                    .owner
                    .map(|user| user.login.unwrap_or_default())
                    .unwrap_or_default(),
                repo_name: repo.name.unwrap_or_default(),
                ssh_url: repo
                    .ssh_url
                    .expect("ssh url to be set for a gitea repository"),
            })
            .collect())
    }

    fn get_domain(api: &str) -> anyhow::Result<String> {
        let url = Url::parse(api)?;
        let provider = url.domain().unwrap_or("gitea");

        Ok(provider.into())
    }

    #[tracing::instrument(skip(self))]
    async fn list_repositories_for_current_user_with_page(
        &self,
        user: &str,
        config: &Configuration,
        page: usize,
    ) -> anyhow::Result<Vec<gitea_rs::models::Repository>> {
        let repos =
            gitea_rs::apis::user_api::user_current_list_repos(config, Some(page as i32), None)
                .await
                .context("failed to fetch repos for users")?;

        Ok(repos)
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_repositories_for_user(
        &self,
        user: &str,
        api: &str,
        access_token: Option<&GiteaAccessToken>,
    ) -> anyhow::Result<Vec<super::Repository>> {
        tracing::debug!("fetching gitea repositories for user");

        let config = self.get_config(api, access_token)?;

        let mut repositories = Vec::new();
        let mut page = 1;
        loop {
            let mut repos = self
                .list_repositories_for_user_with_page(user, &config, page)
                .await?;

            if repos.is_empty() {
                break;
            }

            repositories.append(&mut repos);
            page += 1;
        }

        let provider = &Self::get_domain(api)?;

        Ok(repositories
            .into_iter()
            .map(|repo| super::Repository {
                provider: provider.into(),
                owner: repo
                    .owner
                    .map(|user| user.login.unwrap_or_default())
                    .unwrap_or_default(),
                repo_name: repo.name.unwrap_or_default(),
                ssh_url: repo
                    .ssh_url
                    .expect("ssh url to be set for gitea repository"),
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_repositories_for_user_with_page(
        &self,
        user: &str,
        config: &Configuration,
        page: usize,
    ) -> anyhow::Result<Vec<gitea_rs::models::Repository>> {
        let repos =
            gitea_rs::apis::user_api::user_list_repos(config, user, Some(page as i32), None)
                .await
                .context("failed to fetch repos for users")?;

        Ok(repos)
    }

    #[tracing::instrument]
    pub async fn list_repositories_for_organisation(
        &self,
        organisation: &str,
        api: &str,
        access_token: Option<&GiteaAccessToken>,
    ) -> anyhow::Result<Vec<super::Repository>> {
        let config = self.get_config(api, access_token)?;

        let mut repositories = Vec::new();
        let mut page = 1;
        loop {
            let mut repos = self
                .list_repositories_for_organisation_with_page(organisation, &config, page)
                .await?;

            if repos.is_empty() {
                break;
            }

            repositories.append(&mut repos);
            page += 1;
        }

        let provider = &Self::get_domain(api)?;

        Ok(repositories
            .into_iter()
            .map(|repo| super::Repository {
                provider: provider.into(),
                owner: repo
                    .owner
                    .map(|user| user.login.unwrap_or_default())
                    .unwrap_or_default(),
                repo_name: repo.name.unwrap_or_default(),
                ssh_url: repo
                    .ssh_url
                    .expect("ssh url to be set for gitea repository"),
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    pub async fn list_repositories_for_organisation_with_page(
        &self,
        organisation: &str,
        config: &Configuration,
        page: usize,
    ) -> anyhow::Result<Vec<gitea_rs::models::Repository>> {
        let repos = gitea_rs::apis::organization_api::org_list_repos(
            config,
            organisation,
            Some(page as i32),
            None,
        )
        .await
        .context("failed to fetch repos for users")?;

        Ok(repos)
    }

    fn get_config(
        &self,
        api: &str,
        access_token: Option<&GiteaAccessToken>,
    ) -> anyhow::Result<Configuration> {
        let mut config = gitea_rs::apis::configuration::Configuration::new();
        config.base_path = api.into();
        match access_token {
            Some(GiteaAccessToken::Env { env }) => {
                let token =
                    std::env::var(env).context(format!("{env} didn't have a valid value"))?;

                config.basic_auth = Some(("".into(), Some(token)));
            }
            Some(GiteaAccessToken::Direct(var)) => {
                config.bearer_access_token = Some(var.to_owned());
            }
            None => {}
        }

        Ok(config)
    }
}

pub trait GiteaProviderApp {
    fn gitea_provider(&self) -> GiteaProvider;
}

impl GiteaProviderApp for &'static App {
    fn gitea_provider(&self) -> GiteaProvider {
        GiteaProvider::new(self)
    }
}
