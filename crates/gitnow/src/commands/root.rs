use std::{collections::BTreeMap, io::IsTerminal};

use crate::{
    app::App,
    cache::CacheApp,
    components::inline_command::InlineCommand,
    fuzzy_matcher::{FuzzyMatcher, FuzzyMatcherApp},
    git_clone::GitCloneApp,
    git_provider::Repository,
    interactive::InteractiveApp,
    projects_list::ProjectsListApp,
    shell::ShellApp,
};

#[derive(Debug, Clone)]
pub struct RootCommand {
    app: &'static App,
}

impl RootCommand {
    pub fn new(app: &'static App) -> Self {
        Self { app }
    }

    pub async fn execute(
        &mut self,
        search: Option<impl Into<String>>,
        cache: bool,
        clone: bool,
        shell: bool,
        force_refresh: bool,
        force_cache_update: bool,
    ) -> anyhow::Result<()> {
        tracing::debug!("executing");

        let repositories = if !force_cache_update {
            if cache {
                match self.app.cache().get().await? {
                    Some(repos) => repos,
                    None => {
                        tracing::info!("finding repositories...");
                        let repositories = self.app.projects_list().get_projects().await?;

                        self.app.cache().update(&repositories).await?;

                        repositories
                    }
                }
            } else {
                self.app.projects_list().get_projects().await?
            }
        } else {
            tracing::info!("forcing cache update...");
            let repositories = self.app.projects_list().get_projects().await?;

            self.app.cache().update(&repositories).await?;

            repositories
        };

        let repo = match search {
            Some(needle) => {
                let matched_repos = self
                    .app
                    .fuzzy_matcher()
                    .match_repositories(&needle.into(), &repositories);

                let repo = matched_repos
                    .first()
                    .ok_or(anyhow::anyhow!("failed to find repository"))?;
                tracing::debug!("selected repo: {}", repo.to_rel_path().display());

                repo.to_owned()
            }
            None => {
                let repo = self
                    .app
                    .interactive()
                    .interactive_search(&repositories)?
                    .ok_or(anyhow::anyhow!("failed to find a repository"))?;

                tracing::debug!("selected repo: {}", repo.to_rel_path().display());

                repo
            }
        };

        let project_path = self
            .app
            .config
            .settings
            .projects
            .directory
            .join(repo.to_rel_path());
        if !project_path.exists() {
            if clone {
                let git_clone = self.app.git_clone();

                if std::io::stdout().is_terminal() && shell {
                    let mut wrap_cmd =
                        InlineCommand::new(format!("cloning: {}", repo.to_rel_path().display()));
                    let repo = repo.clone();
                    wrap_cmd
                        .execute(move || async move {
                            git_clone.clone_repo(&repo, force_refresh).await?;

                            Ok(())
                        })
                        .await?;
                } else {
                    eprintln!("cloning repository...");
                    git_clone.clone_repo(&repo, force_refresh).await?;
                }
            } else {
                tracing::info!("skipping clone for repo: {}", &repo.to_rel_path().display());
            }
        } else {
            tracing::info!("repository already exists");
        }

        if shell {
            self.app.shell().spawn_shell(&repo).await?;
        } else {
            tracing::info!("skipping shell for repo: {}", &repo.to_rel_path().display());
            println!(
                "{}",
                self.app
                    .config
                    .settings
                    .projects
                    .directory
                    .join(repo.to_rel_path())
                    .display()
            );
        }

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

pub trait RepositoryMatcher {
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
