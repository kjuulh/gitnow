use anyhow::Context;
use octocrab::{
    auth::Auth,
    models::{hooks::Config, Repository},
    params::repos::Sort,
    NoSvc, Octocrab, Page,
};

use crate::{app::App, config::GitHubAccessToken};

pub struct GitHubProvider {
    #[allow(dead_code)]
    app: &'static App,
}

impl GitHubProvider {
    pub fn new(app: &'static App) -> GitHubProvider {
        GitHubProvider { app }
    }

    pub async fn list_repositories_for_current_user(
        &self,
        url: Option<&String>,
        access_token: &GitHubAccessToken,
    ) -> anyhow::Result<Vec<super::Repository>> {
        tracing::debug!("fetching github repositories for current user");

        let client = self.get_client(url, access_token)?;

        let current_page = client
            .current()
            .list_repos_for_authenticated_user()
            .type_("all")
            .per_page(100)
            .sort("full_name")
            .send()
            .await?;

        let repos = self.unfold_pages(client, current_page).await?;

        Ok(repos
            .into_iter()
            .filter_map(|repo| {
                Some(super::Repository {
                    provider: self.get_url(url),
                    owner: repo.owner.map(|su| su.login)?,
                    repo_name: repo.name,
                    ssh_url: repo.ssh_url?,
                })
            })
            .collect())
    }

    pub async fn list_repositories_for_user(
        &self,
        user: &str,
        url: Option<&String>,
        access_token: &GitHubAccessToken,
    ) -> anyhow::Result<Vec<super::Repository>> {
        tracing::debug!(user = user, "fetching github repositories for user");

        let client = self.get_client(url, access_token)?;

        let current_page = client
            .users(user)
            .repos()
            .r#type(octocrab::params::users::repos::Type::All)
            .sort(Sort::FullName)
            .per_page(100)
            .send()
            .await?;

        let repos = self.unfold_pages(client, current_page).await?;

        Ok(repos
            .into_iter()
            .filter_map(|repo| {
                Some(super::Repository {
                    provider: self.get_url(url),
                    owner: repo.owner.map(|su| su.login)?,
                    repo_name: repo.name,
                    ssh_url: repo.ssh_url?,
                })
            })
            .collect())
    }

    pub async fn list_repositories_for_organisation(
        &self,
        organisation: &str,
        url: Option<&String>,
        access_token: &GitHubAccessToken,
    ) -> anyhow::Result<Vec<super::Repository>> {
        tracing::debug!(
            organisation = organisation,
            "fetching github repositories for organisation"
        );

        let client = self.get_client(url, access_token)?;

        let current_page = client
            .orgs(organisation)
            .list_repos()
            .repo_type(Some(octocrab::params::repos::Type::All))
            .sort(Sort::FullName)
            .per_page(100)
            .send()
            .await?;

        let repos = self.unfold_pages(client, current_page).await?;

        Ok(repos
            .into_iter()
            .filter_map(|repo| {
                Some(super::Repository {
                    provider: self.get_url(url),
                    owner: repo.owner.map(|su| su.login)?,
                    repo_name: repo.name,
                    ssh_url: repo.ssh_url?,
                })
            })
            .collect())
    }

    async fn unfold_pages(
        &self,
        client: octocrab::Octocrab,
        page: Page<Repository>,
    ) -> anyhow::Result<Vec<Repository>> {
        let mut current_page = page;

        let mut repos = current_page.take_items();
        while let Ok(Some(mut new_page)) = client.get_page(&current_page.next).await {
            repos.extend(new_page.take_items());

            current_page = new_page;
        }

        Ok(repos)
    }

    fn get_url(&self, url: Option<&String>) -> String {
        let default_domain = "github.com".to_string();

        if let Some(url) = url {
            let Some(url) = url::Url::parse(url).ok() else {
                return default_domain;
            };

            let Some(domain) = url.domain().map(|d| d.to_string()) else {
                return default_domain;
            };

            domain
        } else {
            default_domain
        }
    }

    fn get_client(
        &self,
        url: Option<&String>,
        access_token: &GitHubAccessToken,
    ) -> anyhow::Result<Octocrab> {
        let client = octocrab::Octocrab::builder()
            .personal_token(match access_token {
                GitHubAccessToken::Direct(token) => token.to_owned(),
                GitHubAccessToken::Env { env } => std::env::var(env)?,
            })
            .build()?;

        Ok(client)
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
