use nucleo_matcher::{pattern::Pattern, Matcher, Utf32Str};

use crate::{app::App, cache::CacheApp, projects_list::ProjectsListApp};

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

        let haystack = repositories
            .iter()
            .map(|r| r.to_rel_path().display().to_string());

        let needle = match search {
            Some(needle) => needle.into(),
            None => todo!(),
        };

        let pattern = Pattern::new(
            &needle,
            nucleo_matcher::pattern::CaseMatching::Ignore,
            nucleo_matcher::pattern::Normalization::Smart,
            nucleo_matcher::pattern::AtomKind::Fuzzy,
        );
        let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
        let res = pattern.match_list(haystack, &mut matcher);

        let res = res.iter().take(10).rev().collect::<Vec<_>>();

        for (repo, _score) in res {
            tracing::debug!("repo: {:?}", repo);
        }

        tracing::info!("amount of repos fetched {}", repositories.len());

        Ok(())
    }
}
