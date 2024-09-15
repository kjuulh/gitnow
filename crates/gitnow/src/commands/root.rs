use crate::{
    app::App, cache::CacheApp, fuzzy_matcher::FuzzyMatcherApp, projects_list::ProjectsListApp,
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

        let haystack = repositories
            .iter()
            .map(|r| r.to_rel_path().display().to_string())
            .collect::<Vec<_>>();

        let haystack = haystack.as_str_vec();

        let res = self.app.fuzzy_matcher().match_pattern(&needle, &haystack);

        let res = res.iter().take(10).rev().collect::<Vec<_>>();

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
