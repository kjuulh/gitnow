/// The `list` subcommand prints the known repository set non-interactively.
///
/// Repositories are gitnow's unprefixed noun — `gitnow <query>` searches them — so listing
/// them is `gitnow list`, and `gitnow project list` stays the projects-scoped sibling.
///
/// Everything else that reads the repository set either needs a TTY (the picker) or collapses
/// it to a single top match (the root command's fuzzy path), which leaves no way to enumerate
/// from a script or another program's completion UI. This is that way.
use std::path::PathBuf;

use crate::{
    app::App, cache::load_repositories, commands::root::RepositoryMatcher,
    fuzzy_matcher::FuzzyMatcherApp, git_provider::Repository,
};

#[derive(clap::Parser)]
pub struct ListCommand {
    /// Repository query terms. All terms must match.
    #[arg(value_name = "QUERY", num_args = 0..)]
    search: Vec<String>,

    /// Output as JSON
    #[arg(long = "json", default_value = "false")]
    json: bool,

    /// Only repositories that are already cloned locally
    #[arg(long = "cloned", default_value = "false")]
    cloned: bool,

    #[arg(long = "no-cache", default_value = "false")]
    no_cache: bool,
}

impl ListCommand {
    pub async fn execute(&self, app: &'static App) -> anyhow::Result<()> {
        let repositories = load_repositories(app, !self.no_cache).await?;

        // Same matcher as the root command, so `gitnow list foo` and `gitnow foo` agree on
        // what "foo" means — this just keeps every hit instead of the best one.
        let matched = match (!self.search.is_empty()).then(|| self.search.join(" ")) {
            Some(needle) => app
                .fuzzy_matcher()
                .match_repositories(&needle, &repositories),
            None => repositories,
        };

        let root = &app.config.settings.projects.directory;
        let entries: Vec<(Repository, PathBuf, bool)> = matched
            .into_iter()
            .map(|repo| {
                let path = root.join(repo.to_rel_path());
                let cloned = path.exists();
                (repo, path, cloned)
            })
            .filter(|(_, _, cloned)| *cloned || !self.cloned)
            .collect();

        if self.json {
            let out: Vec<serde_json::Value> = entries
                .iter()
                .map(|(repo, path, cloned)| {
                    serde_json::json!({
                        "provider": repo.provider,
                        "owner": repo.owner,
                        "repo_name": repo.repo_name,
                        "ssh_url": repo.ssh_url,
                        // Where the repo lives once cloned, whether or not it is yet: the
                        // path is derived, so a consumer needs `cloned` to tell them apart.
                        "path": path.display().to_string(),
                        "cloned": cloned,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            // Bare relative paths, one per line: the same label the picker matches on, so
            // the output pipes straight back into `gitnow` or into fzf.
            for (repo, _, _) in &entries {
                println!("{}", repo.to_rel_path().display());
            }
        }

        Ok(())
    }
}
