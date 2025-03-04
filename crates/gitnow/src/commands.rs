pub mod root;
pub mod shell;
pub mod update;
pub mod clone {

    use std::sync::Arc;

    use futures::future::join_all;
    use regex::Regex;

    use crate::{
        app::App, cache::CacheApp, custom_command::CustomCommandApp, git_clone::GitCloneApp,
    };

    #[derive(clap::Parser)]
    pub struct CloneCommand {
        #[arg(long = "search")]
        search: String,
    }

    impl CloneCommand {
        pub async fn execute(&mut self, app: &'static App) -> anyhow::Result<()> {
            let repos = app.cache().get().await?.ok_or(anyhow::anyhow!(
                "failed to get cache, do a gitnow update first"
            ))?;

            let search = Regex::new(&self.search)?;

            let filtered_repos = repos
                .iter()
                .filter(|r| search.is_match(&r.to_rel_path().display().to_string()))
                .collect::<Vec<_>>();

            let concurrency_limit = Arc::new(tokio::sync::Semaphore::new(5));
            let mut handles = Vec::default();
            for repo in filtered_repos {
                let config = app.config.clone();
                let custom_command = app.custom_command();
                let git_clone = app.git_clone();
                let repo = repo.clone();
                let concurrency = Arc::clone(&concurrency_limit);

                let handle = tokio::spawn(async move {
                    let permit = concurrency.acquire().await?;

                    let project_path = config.settings.projects.directory.join(repo.to_rel_path());
                    if !project_path.exists() {
                        eprintln!("cloning repository: {}", repo.to_rel_path().display());
                        git_clone.clone_repo(&repo, false).await?;

                        custom_command
                            .execute_post_clone_command(&project_path)
                            .await?;
                    }

                    drop(permit);

                    Ok::<(), anyhow::Error>(())
                });

                handles.push(handle);
            }

            let res = join_all(handles).await;

            for res in res {
                match res {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        tracing::error!("failed to clone repo: {}", e);
                        anyhow::bail!(e)
                    }
                    Err(e) => {
                        tracing::error!("failed to clone repo: {}", e);
                        anyhow::bail!(e)
                    }
                }
            }

            Ok(())
        }
    }
}
