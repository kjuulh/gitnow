use std::collections::BTreeMap;

use crate::{
    app::App,
    cache::CacheApp,
    fuzzy_matcher::{FuzzyMatcher, FuzzyMatcherApp},
    git_provider::Repository,
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

    pub async fn execute(&mut self, search: Option<impl Into<String>>) -> anyhow::Result<()> {
        tracing::debug!("executing");

        let repositories = match self.app.cache().get().await? {
            Some(repos) => repos,
            None => {
                tracing::info!("finding repositories...");
                let repositories = self.app.projects_list().get_projects().await?;

                self.app.cache().update(&repositories).await?;

                repositories
            }
        };
        let needle = match search {
            Some(needle) => needle.into(),
            None => todo!(),
        };

        let matched_repos = self
            .app
            .fuzzy_matcher()
            .match_repositories(&needle, &repositories);
        let res = matched_repos.iter().take(10).rev().collect::<Vec<_>>();

        for repo in res {
            tracing::debug!("repo: {:?}", repo);
        }

        tracing::info!("amount of repos fetched {}", repositories.len());

        Ok(())
    }
}

trait StringExt {
    fn as_str_vec(&self) -> Vec<&str>;
}

impl StringExt for Vec<String> {
    fn as_str_vec(&self) -> Vec<&str> {
        self.iter().map(|r| r.as_ref()).collect()
    }
}

impl StringExt for Vec<&String> {
    fn as_str_vec(&self) -> Vec<&str> {
        self.iter().map(|r| r.as_ref()).collect()
    }
}

trait RepositoryMatcher {
    fn match_repositories(&self, pattern: &str, repositories: &[Repository]) -> Vec<Repository>;
}

impl RepositoryMatcher for FuzzyMatcher {
    fn match_repositories(&self, pattern: &str, repositories: &[Repository]) -> Vec<Repository> {
        let haystack = repositories
            .iter()
            .map(|r| (r.to_rel_path().display().to_string(), r))
            .collect::<BTreeMap<_, _>>();
        let haystack_keys = haystack.keys().collect::<Vec<_>>();
        let haystack_keys = haystack_keys.as_str_vec();

        let res = self.match_pattern(pattern, &haystack_keys);

        let matched_repos = res
            .into_iter()
            .filter_map(|repo_key| haystack.get(repo_key).map(|r| (*r).to_owned()))
            .collect::<Vec<_>>();

        matched_repos
    }
}
