use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{
    app::App,
    cache::load_repositories,
    custom_command::CustomCommandApp,
    interactive::{InteractiveApp, Searchable},
    shell::ShellApp,
    template_command,
};

#[derive(clap::Parser)]
pub struct ProjectCommand {
    #[command(subcommand)]
    command: Option<ProjectSubcommand>,

    /// Search string to filter existing projects
    #[arg()]
    search: Option<String>,

    /// Skip spawning a shell in the project directory
    #[arg(long = "no-shell", default_value = "false")]
    no_shell: bool,
}

#[derive(clap::Subcommand)]
enum ProjectSubcommand {
    /// Create a new project with selected repositories
    Create(ProjectCreateCommand),
    /// Add repositories to an existing project
    Add(ProjectAddCommand),
    /// Delete an existing project
    Delete(ProjectDeleteCommand),
}

#[derive(clap::Parser)]
pub struct ProjectCreateCommand {
    /// Project name (will be used as directory name)
    #[arg()]
    name: Option<String>,

    /// Bootstrap from a template in the templates directory
    #[arg(long = "template", short = 't')]
    template: Option<String>,

    /// Skip cache when fetching repositories
    #[arg(long = "no-cache", default_value = "false")]
    no_cache: bool,

    /// Skip spawning a shell in the project directory
    #[arg(long = "no-shell", default_value = "false")]
    no_shell: bool,
}

#[derive(clap::Parser)]
pub struct ProjectAddCommand {
    /// Project name to add repositories to
    #[arg()]
    name: Option<String>,

    /// Skip cache when fetching repositories
    #[arg(long = "no-cache", default_value = "false")]
    no_cache: bool,
}

#[derive(clap::Parser)]
pub struct ProjectDeleteCommand {
    /// Project name to delete
    #[arg()]
    name: Option<String>,

    /// Skip confirmation prompt
    #[arg(long = "force", short = 'f', default_value = "false")]
    force: bool,
}

// --- Shared helpers ---

/// A named directory entry usable in interactive search.
#[derive(Clone)]
struct DirEntry {
    name: String,
    path: PathBuf,
}

impl Searchable for DirEntry {
    fn display_label(&self) -> String {
        self.name.clone()
    }
}

/// Resolve a config directory path, expanding `~` to the home directory.
/// Falls back to `default` if the config value is `None`.
fn resolve_dir(configured: Option<&str>, default: &str) -> PathBuf {
    if let Some(dir) = configured {
        let path = PathBuf::from(dir);
        if let Ok(stripped) = path.strip_prefix("~") {
            return dirs::home_dir().unwrap_or_default().join(stripped);
        }
        return path;
    }
    dirs::home_dir().unwrap_or_default().join(default)
}

fn get_projects_dir(app: &'static App) -> PathBuf {
    let configured = app
        .config
        .settings
        .project
        .as_ref()
        .and_then(|p| p.directory.as_deref());
    resolve_dir(configured, ".gitnow/projects")
}

fn get_templates_dir(app: &'static App) -> PathBuf {
    let configured = app
        .config
        .settings
        .project
        .as_ref()
        .and_then(|p| p.templates_directory.as_deref());
    resolve_dir(configured, ".gitnow/templates")
}

/// List subdirectories of `dir` as `DirEntry` items, sorted by name.
fn list_subdirectories(dir: &Path) -> anyhow::Result<Vec<DirEntry>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path(),
            });
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Clone selected repositories concurrently into `target_dir`.
async fn clone_repos_into(
    app: &'static App,
    repos: &[crate::git_provider::Repository],
    target_dir: &Path,
) -> anyhow::Result<()> {
    let clone_template = app
        .config
        .settings
        .clone_command
        .as_deref()
        .unwrap_or(template_command::DEFAULT_CLONE_COMMAND);

    let concurrency_limit = Arc::new(tokio::sync::Semaphore::new(5));
    let mut handles = Vec::new();

    for repo in repos {
        let repo = repo.clone();
        let target_dir = target_dir.to_path_buf();
        let clone_template = clone_template.to_string();
        let concurrency = Arc::clone(&concurrency_limit);
        let custom_command = app.custom_command();

        let handle = tokio::spawn(async move {
            let _permit = concurrency.acquire().await?;

            let clone_path = target_dir.join(&repo.repo_name);

            if clone_path.exists() {
                eprintln!("  {} already exists, skipping", repo.repo_name);
                return Ok::<(), anyhow::Error>(());
            }

            eprintln!("  cloning {}...", repo.to_rel_path().display());

            let path_str = clone_path.display().to_string();
            let context = HashMap::from([
                ("ssh_url", repo.ssh_url.as_str()),
                ("path", path_str.as_str()),
            ]);

            let output = template_command::render_and_execute(&clone_template, context).await?;

            if !output.status.success() {
                let stderr = std::str::from_utf8(&output.stderr).unwrap_or_default();
                anyhow::bail!("failed to clone {}: {}", repo.repo_name, stderr);
            }

            custom_command
                .execute_post_clone_command(&clone_path)
                .await?;

            Ok(())
        });

        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    for res in results {
        match res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                tracing::error!("clone error: {}", e);
                eprintln!("error: {}", e);
            }
            Err(e) => {
                tracing::error!("task error: {}", e);
                eprintln!("error: {}", e);
            }
        }
    }

    Ok(())
}

/// Helper to select an existing project, either by name or interactively.
fn select_project(
    app: &'static App,
    name: Option<String>,
    projects: &[DirEntry],
) -> anyhow::Result<DirEntry> {
    match name {
        Some(name) => projects
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("project '{}' not found", name))
            .cloned(),
        None => app
            .interactive()
            .interactive_search_items(projects)?
            .ok_or_else(|| anyhow::anyhow!("no project selected")),
    }
}

// --- Command implementations ---

impl ProjectCommand {
    pub async fn execute(&mut self, app: &'static App) -> anyhow::Result<()> {
        match self.command.take() {
            Some(ProjectSubcommand::Create(mut create)) => create.execute(app).await,
            Some(ProjectSubcommand::Add(mut add)) => add.execute(app).await,
            Some(ProjectSubcommand::Delete(mut delete)) => delete.execute(app).await,
            None => self.open_existing(app).await,
        }
    }

    async fn open_existing(&self, app: &'static App) -> anyhow::Result<()> {
        let projects_dir = get_projects_dir(app);
        let projects = list_subdirectories(&projects_dir)?;

        if projects.is_empty() {
            anyhow::bail!(
                "no projects found in {}. Use 'gitnow project create' to create one.",
                projects_dir.display()
            );
        }

        let project = match &self.search {
            Some(needle) => {
                let matched = projects
                    .iter()
                    .find(|p| p.name.contains(needle.as_str()))
                    .or_else(|| {
                        projects.iter().find(|p| {
                            p.name
                                .to_lowercase()
                                .contains(&needle.to_lowercase())
                        })
                    })
                    .ok_or(anyhow::anyhow!(
                        "no project matching '{}' found",
                        needle
                    ))?
                    .clone();
                matched
            }
            None => app
                .interactive()
                .interactive_search_items(&projects)?
                .ok_or(anyhow::anyhow!("no project selected"))?,
        };

        if !self.no_shell {
            app.shell().spawn_shell_at(&project.path).await?;
        } else {
            println!("{}", project.path.display());
        }

        Ok(())
    }
}

impl ProjectCreateCommand {
    async fn execute(&mut self, app: &'static App) -> anyhow::Result<()> {
        let name = match self.name.take() {
            Some(n) => n,
            None => {
                eprint!("Project name: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let trimmed = input.trim().to_string();
                if trimmed.is_empty() {
                    anyhow::bail!("project name cannot be empty");
                }
                trimmed
            }
        };

        let dir_name = name
            .replace(' ', "-")
            .replace('/', "-")
            .to_lowercase();

        let projects_dir = get_projects_dir(app);
        let project_path = projects_dir.join(&dir_name);

        if project_path.exists() {
            anyhow::bail!(
                "project '{}' already exists at {}",
                dir_name,
                project_path.display()
            );
        }

        let repositories = load_repositories(app, !self.no_cache).await?;

        eprintln!("Select repositories (Tab to toggle, Enter to confirm):");
        let selected_repos = app
            .interactive()
            .interactive_multi_search(&repositories)?;

        if selected_repos.is_empty() {
            anyhow::bail!("no repositories selected");
        }

        tokio::fs::create_dir_all(&project_path).await?;

        clone_repos_into(app, &selected_repos, &project_path).await?;

        // Apply template if requested
        let templates_dir = get_templates_dir(app);
        let template = match self.template.take() {
            Some(name) => {
                let templates = list_subdirectories(&templates_dir)?;
                Some(
                    templates
                        .into_iter()
                        .find(|t| t.name == name)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "template '{}' not found in {}",
                                name,
                                templates_dir.display()
                            )
                        })?,
                )
            }
            None => {
                let templates = list_subdirectories(&templates_dir)?;
                if !templates.is_empty() {
                    eprintln!("Select a project template (Esc to skip):");
                    app.interactive().interactive_search_items(&templates)?
                } else {
                    None
                }
            }
        };

        if let Some(template) = template {
            eprintln!("  applying template '{}'...", template.name);
            copy_dir_recursive(&template.path, &project_path)?;
        }

        eprintln!(
            "project '{}' created at {} with {} repositories",
            dir_name,
            project_path.display(),
            selected_repos.len()
        );

        if !self.no_shell {
            app.shell().spawn_shell_at(&project_path).await?;
        } else {
            println!("{}", project_path.display());
        }

        Ok(())
    }
}

impl ProjectAddCommand {
    async fn execute(&mut self, app: &'static App) -> anyhow::Result<()> {
        let projects_dir = get_projects_dir(app);
        let projects = list_subdirectories(&projects_dir)?;

        if projects.is_empty() {
            anyhow::bail!(
                "no projects found in {}. Use 'gitnow project create' to create one.",
                projects_dir.display()
            );
        }

        let project = select_project(app, self.name.take(), &projects)?;

        let repositories = load_repositories(app, !self.no_cache).await?;

        eprintln!("Select repositories to add (Tab to toggle, Enter to confirm):");
        let selected_repos = app
            .interactive()
            .interactive_multi_search(&repositories)?;

        if selected_repos.is_empty() {
            anyhow::bail!("no repositories selected");
        }

        clone_repos_into(app, &selected_repos, &project.path).await?;

        eprintln!(
            "added {} repositories to project '{}'",
            selected_repos.len(),
            project.name
        );

        Ok(())
    }
}

impl ProjectDeleteCommand {
    async fn execute(&mut self, app: &'static App) -> anyhow::Result<()> {
        let projects_dir = get_projects_dir(app);
        let projects = list_subdirectories(&projects_dir)?;

        if projects.is_empty() {
            anyhow::bail!("no projects found in {}", projects_dir.display());
        }

        let project = select_project(app, self.name.take(), &projects)?;

        if !self.force {
            eprint!(
                "Delete project '{}' at {}? [y/N] ",
                project.name,
                project.path.display()
            );
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                eprintln!("aborted");
                return Ok(());
            }
        }

        tokio::fs::remove_dir_all(&project.path).await?;
        eprintln!("deleted project '{}'", project.name);

        Ok(())
    }
}
