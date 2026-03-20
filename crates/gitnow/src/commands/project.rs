use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    app::App,
    cache::CacheApp,
    custom_command::CustomCommandApp,
    interactive::{InteractiveApp, Searchable},
    projects_list::ProjectsListApp,
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
    /// Delete an existing project
    Delete(ProjectDeleteCommand),
}

#[derive(clap::Parser)]
pub struct ProjectCreateCommand {
    /// Project name (will be used as directory name)
    #[arg()]
    name: Option<String>,

    /// Skip cache when fetching repositories
    #[arg(long = "no-cache", default_value = "false")]
    no_cache: bool,

    /// Skip spawning a shell in the project directory
    #[arg(long = "no-shell", default_value = "false")]
    no_shell: bool,
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

fn get_projects_dir(app: &'static App) -> PathBuf {
    if let Some(ref project_settings) = app.config.settings.project {
        if let Some(ref dir) = project_settings.directory {
            let path = PathBuf::from(dir);
            if let Ok(stripped) = path.strip_prefix("~") {
                let home = dirs::home_dir().unwrap_or_default();
                return home.join(stripped);
            }
            return path;
        }
    }

    let home = dirs::home_dir().unwrap_or_default();
    home.join(".gitnow").join("projects")
}

#[derive(Clone)]
struct ProjectEntry {
    name: String,
    path: PathBuf,
}

impl Searchable for ProjectEntry {
    fn display_label(&self) -> String {
        self.name.clone()
    }
}

fn list_existing_projects(projects_dir: &PathBuf) -> anyhow::Result<Vec<ProjectEntry>> {
    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();
    for entry in std::fs::read_dir(projects_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            projects.push(ProjectEntry {
                name,
                path: entry.path(),
            });
        }
    }
    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(projects)
}

impl ProjectCommand {
    pub async fn execute(&mut self, app: &'static App) -> anyhow::Result<()> {
        match self.command.take() {
            Some(ProjectSubcommand::Create(mut create)) => create.execute(app).await,
            Some(ProjectSubcommand::Delete(mut delete)) => delete.execute(app).await,
            None => self.open_existing(app).await,
        }
    }

    async fn open_existing(&self, app: &'static App) -> anyhow::Result<()> {
        let projects_dir = get_projects_dir(app);
        let projects = list_existing_projects(&projects_dir)?;

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
                        // fuzzy fallback
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
        // Step 1: Get project name
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

        // Sanitize project name for use as directory
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

        // Step 2: Load repositories
        let repositories = if !self.no_cache {
            match app.cache().get().await? {
                Some(repos) => repos,
                None => {
                    eprintln!("fetching repositories...");
                    let repositories = app.projects_list().get_projects().await?;
                    app.cache().update(&repositories).await?;
                    repositories
                }
            }
        } else {
            app.projects_list().get_projects().await?
        };

        // Step 3: Multi-select repositories
        eprintln!("Select repositories (Tab to toggle, Enter to confirm):");
        let selected_repos = app
            .interactive()
            .interactive_multi_search(&repositories)?;

        if selected_repos.is_empty() {
            anyhow::bail!("no repositories selected");
        }

        // Step 4: Create project directory
        tokio::fs::create_dir_all(&project_path).await?;

        // Step 5: Clone each selected repository into the project directory
        let clone_template = app
            .config
            .settings
            .clone_command
            .as_deref()
            .unwrap_or(template_command::DEFAULT_CLONE_COMMAND);

        let concurrency_limit = Arc::new(tokio::sync::Semaphore::new(5));
        let mut handles = Vec::new();

        for repo in &selected_repos {
            let repo = repo.clone();
            let project_path = project_path.clone();
            let clone_template = clone_template.to_string();
            let concurrency = Arc::clone(&concurrency_limit);
            let custom_command = app.custom_command();

            let handle = tokio::spawn(async move {
                let permit = concurrency.acquire().await?;

                let clone_path = project_path.join(&repo.repo_name);

                if clone_path.exists() {
                    eprintln!("  {} already exists, skipping", repo.repo_name);
                    drop(permit);
                    return Ok::<(), anyhow::Error>(());
                }

                eprintln!("  cloning {}...", repo.to_rel_path().display());

                let path_str = clone_path.display().to_string();
                let context = HashMap::from([
                    ("ssh_url", repo.ssh_url.as_str()),
                    ("path", path_str.as_str()),
                ]);

                let output =
                    template_command::render_and_execute(&clone_template, context).await?;

                if !output.status.success() {
                    let stderr = std::str::from_utf8(&output.stderr).unwrap_or_default();
                    anyhow::bail!("failed to clone {}: {}", repo.repo_name, stderr);
                }

                custom_command
                    .execute_post_clone_command(&clone_path)
                    .await?;

                drop(permit);
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

        eprintln!(
            "project '{}' created at {} with {} repositories",
            dir_name,
            project_path.display(),
            selected_repos.len()
        );

        // Step 6: Enter shell or print path
        if !self.no_shell {
            app.shell().spawn_shell_at(&project_path).await?;
        } else {
            println!("{}", project_path.display());
        }

        Ok(())
    }
}

impl ProjectDeleteCommand {
    async fn execute(&mut self, app: &'static App) -> anyhow::Result<()> {
        let projects_dir = get_projects_dir(app);
        let projects = list_existing_projects(&projects_dir)?;

        if projects.is_empty() {
            anyhow::bail!("no projects found in {}", projects_dir.display());
        }

        let project = match self.name.take() {
            Some(name) => projects
                .iter()
                .find(|p| p.name == name)
                .ok_or(anyhow::anyhow!("project '{}' not found", name))?
                .clone(),
            None => app
                .interactive()
                .interactive_search_items(&projects)?
                .ok_or(anyhow::anyhow!("no project selected"))?,
        };

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
