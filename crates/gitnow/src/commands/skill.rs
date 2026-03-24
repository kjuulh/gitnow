/// The `skill` subcommand outputs a comprehensive, LLM-readable description of
/// everything gitnow can do — commands, flags, configuration, and workflows.

#[derive(clap::Parser)]
pub struct SkillCommand {}

impl SkillCommand {
    pub async fn execute(&self) -> anyhow::Result<()> {
        print!("{}", SKILL_TEXT);
        Ok(())
    }
}

const SKILL_TEXT: &str = r#"# gitnow — Navigate git projects at the speed of thought

gitnow is a CLI tool for discovering, cloning, and navigating git repositories
from multiple providers (GitHub, Gitea). It maintains a local cache of known
repositories and provides fuzzy-search, interactive selection, worktree
management, and scratch-pad project workspaces.

## Quick reference

```
gitnow [OPTIONS] [SEARCH]            # search/clone/open a repository
gitnow update                        # refresh the local repository cache
gitnow clone --search <REGEX>        # batch-clone repositories matching a pattern
gitnow worktree [SEARCH] [OPTIONS]   # create and enter a git worktree for a branch
gitnow project [SEARCH] [OPTIONS]    # open an existing scratch-pad project
gitnow project create [NAME]         # create a new multi-repo project
gitnow project add [NAME]            # add repositories to a project
gitnow project delete [NAME]         # delete a project
gitnow project list [OPTIONS]        # list all projects and their repos
gitnow init zsh                      # print zsh shell integration script
gitnow skill                         # print this reference (you are here)
```

## Commands in detail

### Default (no subcommand)

```
gitnow [OPTIONS] [SEARCH]
```

Search for a repository, optionally clone it, and open a shell inside it.

- If SEARCH is provided, fuzzy-matches against cached repositories.
- If omitted, opens an interactive fuzzy-search picker.
- Clones the repository if it does not exist locally.
- Spawns a sub-shell in the repository directory.

**Flags:**
| Flag                  | Description                                              |
|-----------------------|----------------------------------------------------------|
| `--no-cache`          | Skip reading from the local cache; fetch fresh data      |
| `--no-clone`          | Do not clone the repository if it is missing locally     |
| `--no-shell`          | Print the path instead of spawning a shell               |
| `--force-refresh`     | Force a fresh clone even if the repo already exists      |
| `--force-cache-update`| Update the cache before searching                        |
| `--chooser-file PATH` | Write selected path to this file (implies --no-shell)   |
| `-c, --config PATH`  | Path to config file (global flag)                        |

**Environment variables:**
- `GITNOW_CONFIG` — path to config file (overrides default)
- `GITNOW_CHOOSER_FILE` — equivalent to `--chooser-file`

---

### `gitnow update`

Fetch all repositories from configured providers and update the local cache.
Should be run periodically or after adding new providers/organisations.

No flags.

---

### `gitnow clone --search <REGEX>`

Batch-clone all repositories whose relative path matches the given regex.
Clones up to 5 repositories concurrently. Skips repos that already exist locally.

**Required flags:**
| Flag               | Description                                |
|--------------------|--------------------------------------------|
| `--search <REGEX>` | Regular expression to match repository paths |

---

### `gitnow worktree [SEARCH] [OPTIONS]`

Create a git worktree for a specific branch of a repository. This is useful for
working on multiple branches simultaneously without switching.

**Workflow:**
1. Select a repository (fuzzy search or interactive picker)
2. Bare-clone the repository if not already present
3. List remote branches
4. Select a branch (interactive picker, or `--branch`)
5. Create a worktree directory at `<project>/<sanitized-branch>/`
6. Spawn a shell in the worktree (or write path to chooser file)

**Flags:**
| Flag              | Description                                    |
|-------------------|------------------------------------------------|
| `[SEARCH]`        | Optional search string to pre-filter repos     |
| `-b, --branch`    | Branch name (skips interactive branch picker)   |
| `--no-cache`      | Skip the local cache                           |
| `--no-shell`      | Print path instead of spawning a shell         |

---

### `gitnow project [SEARCH] [OPTIONS]`

Manage scratch-pad projects — directories containing multiple cloned repositories,
optionally bootstrapped from a template.

When called without a subcommand, opens an existing project (interactive picker
or fuzzy match on SEARCH).

**Flags:**
| Flag         | Description                              |
|--------------|------------------------------------------|
| `[SEARCH]`   | Filter existing projects by name         |
| `--no-shell` | Print path instead of spawning a shell   |

#### `gitnow project create [NAME] [OPTIONS]`

Create a new project directory, select repositories to clone into it,
and optionally apply a template.

| Flag                | Description                                          |
|---------------------|------------------------------------------------------|
| `[NAME]`            | Project name (prompted if omitted)                   |
| `-r, --repos`       | Repositories to include (fuzzy-matched). Repeatable: `--repos foo --repos bar`. Skips interactive picker when provided. |
| `-t, --template`    | Template name to bootstrap from                      |
| `--no-template`     | Skip template selection entirely                     |
| `--no-cache`        | Skip local cache when listing repos                  |
| `--no-shell`        | Print path instead of spawning a shell               |

Templates live in `~/.gitnow/templates/` (or the configured directory). Each
subdirectory is a template; its contents are copied into the new project.

**Non-interactive usage:**
```
gitnow project create my-feature --repos repo-a --repos repo-b --no-template --no-shell
```

#### `gitnow project add [NAME] [OPTIONS]`

Add more repositories to an existing project.

| Flag         | Description                              |
|--------------|------------------------------------------|
| `[NAME]`     | Project name (interactive if omitted)    |
| `-r, --repos`| Repositories to add (fuzzy-matched). Repeatable. Skips interactive picker when provided. |
| `--no-cache` | Skip local cache when listing repos      |

**Non-interactive usage:**
```
gitnow project add my-feature --repos repo-c --repos repo-d
```

#### `gitnow project list [OPTIONS]`

List all projects and optionally show their repositories.

| Flag         | Description                              |
|--------------|------------------------------------------|
| `--repos`    | Show repository details for each project |
| `--json`     | Output as JSON                           |

#### `gitnow project delete [NAME] [OPTIONS]`

Delete a project directory.

| Flag          | Description                          |
|---------------|--------------------------------------|
| `[NAME]`      | Project name (interactive if omitted)|
| `-f, --force` | Skip confirmation prompt             |

---

### `gitnow init zsh`

Print a zsh shell integration script to stdout. Typically used as:

```zsh
eval "$(gitnow init zsh)"
```

This provides a shell function that changes directory after gitnow exits,
using the chooser-file mechanism.

---

### `gitnow skill`

Print this LLM-readable reference document to stdout.

---

## Configuration

Config file location (in priority order):
1. `--config` / `-c` flag
2. `GITNOW_CONFIG` environment variable
3. `~/.config/gitnow/gitnow.toml`

### Config file format (TOML)

```toml
[settings]
# Where repositories are cloned to (default: ~/git)
projects = { directory = "~/git" }

# Custom clone command (minijinja template)
# Available variables: {{ ssh_url }}, {{ path }}
# Default: "git clone {{ ssh_url }} {{ path }}"
clone_command = "git clone {{ ssh_url }} {{ path }}"

# Commands to run after cloning a repository
post_clone_command = "echo 'cloned!'"
# or as a list:
# post_clone_command = ["cmd1", "cmd2"]

# Commands to run when opening an already-cloned repository
post_update_command = "git fetch --prune"

[settings.cache]
# Where the cache is stored (default: ~/.cache/gitnow)
location = "~/.cache/gitnow"

# Cache duration (default: 7 days). Set to false to disable.
duration = { days = 7, hours = 0, minutes = 0 }

[settings.worktree]
# Custom worktree commands (minijinja templates)
clone_command = "git clone --bare {{ ssh_url }} {{ bare_path }}"
add_command = "git -C {{ bare_path }} worktree add {{ worktree_path }} {{ branch }}"
list_branches_command = "git -C {{ bare_path }} branch -r --format=%(refname:short)"

[settings.project]
# Where scratch-pad projects are stored (default: ~/.gitnow/projects)
directory = "~/.gitnow/projects"
# Where project templates live (default: ~/.gitnow/templates)
templates_directory = "~/.gitnow/templates"

# --- Providers ---

[[providers.github]]
access_token = "ghp_..."           # or { env = "GITHUB_TOKEN" }
current_user = "your-username"     # optional, for user-specific repos
users = ["user1"]                  # fetch repos for these users
organisations = ["org1", "org2"]   # fetch repos for these orgs
url = "https://api.github.com"     # optional, for GitHub Enterprise

[[providers.gitea]]
url = "https://gitea.example.com/api/v1"
access_token = "token"             # or { env = "GITEA_TOKEN" }
current_user = "your-username"
users = ["user1"]
organisations = ["org1"]
```

Multiple provider entries are supported — gitnow aggregates repositories from all of them.

## Typical workflows

### First-time setup
1. Create `~/.config/gitnow/gitnow.toml` with at least one provider
2. Run `gitnow update` to populate the cache
3. Run `gitnow` to interactively search and clone a repo

### Daily use
- `gitnow <partial-name>` — jump to a repo by fuzzy name
- `gitnow worktree <repo> -b feature-x` — start work on a branch in a worktree
- `gitnow project create my-feature` — set up a multi-repo workspace

### Shell integration (zsh)
Add to `.zshrc`:
```zsh
eval "$(gitnow init zsh)"
```
This wraps gitnow so that selecting a repository changes your current shell's
working directory (instead of spawning a sub-shell).
"#;
