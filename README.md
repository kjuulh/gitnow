# Git Now

> https://gitnow.kjuulh.io/

Git Now is a utility for easily navigating git projects from common upstream providers. Search, Download, and Enter projects as quickly as you can type.

![example gif](./assets/gifs/example.gif)

## Installation

```bash
cargo (b)install gitnow

# You can either use gitnow directly (and use spawned shell sessions)
gitnow

# Or install gitnow scripts (in your .bashrc, .zshrc) this will use native shell commands to move you around
eval $(gitnow init zsh)
git-now # Long 
gn # Short alias
```

## Reasoning

How many steps do you normally do to download a project?

1. Navigate to github.com
2. Search in your org for the project
3. Find the clone url
4. Navigate to your local github repositories path
5. Git clone `<project>` 
6. Enter new project directory

A power user can of course use `gh repo clone` to skip a few steps.

With gitnow

1. `git now`
2. Enter parts of the project name and press enter
3. Your project is automatically downloaded if it doesn't exist in an opinionated path dir, and move you there.

## Configuration

Configuration lives at `~/.config/gitnow/gitnow.toml` (override with `$GITNOW_CONFIG`).

### Custom clone command

By default gitnow uses `git clone`. You can override this with any command using a [minijinja](https://docs.rs/minijinja) template:

```toml
[settings]
# Use jj (Jujutsu) instead of git
clone_command = "jj git clone {{ ssh_url }} {{ path }}"
```

Available template variables: `ssh_url`, `path`.

### Worktrees

gitnow supports git worktrees (or jj workspaces) via the `worktree` subcommand. This uses bare repositories so each branch gets its own directory as a sibling:

```
~/git/github.com/owner/repo/
├── .bare/          # bare clone (git clone --bare)
├── main/           # worktree for main branch
├── feature-login/  # worktree for feature/login branch
└── fix-typo/       # worktree for fix/typo branch
```

Usage:

```bash
# Interactive: pick repo, then pick branch
gitnow worktree

# Pre-filter repo
gitnow worktree myproject

# Specify branch directly
gitnow worktree myproject -b feature/login

# Print worktree path instead of entering a shell
gitnow worktree myproject -b main --no-shell
```

All worktree commands are configurable via minijinja templates:

```toml
[settings.worktree]
# Default: "git clone --bare {{ ssh_url }} {{ bare_path }}"
clone_command = "git clone --bare {{ ssh_url }} {{ bare_path }}"

# Default: "git -C {{ bare_path }} worktree add {{ worktree_path }} {{ branch }}"
add_command = "git -C {{ bare_path }} worktree add {{ worktree_path }} {{ branch }}"

# Default: "git -C {{ bare_path }} branch --format=%(refname:short)"
list_branches_command = "git -C {{ bare_path }} branch --format=%(refname:short)"
```

For jj, you might use:

```toml
[settings]
clone_command = "jj git clone {{ ssh_url }} {{ path }}"

[settings.worktree]
clone_command = "jj git clone {{ ssh_url }} {{ bare_path }}"
add_command = "jj -R {{ bare_path }} workspace add --name {{ branch }} {{ worktree_path }}"
list_branches_command = "jj -R {{ bare_path }} bookmark list -T 'name ++ \"\\n\"'"
```

Available template variables for worktree commands: `bare_path`, `worktree_path`, `branch`, `ssh_url`.
