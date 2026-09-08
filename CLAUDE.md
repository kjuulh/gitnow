# gitnow — usage guide

gitnow discovers, clones, and navigates git repositories from GitHub and Gitea.
It keeps a local cache of every repo it can see, then lets you fuzzy-search that
cache to clone-and-enter a repo in one step, cut a worktree for a branch, or
assemble a throwaway multi-repo workspace.

`README.md` covers what gitnow is and why it exists. This file covers **how to
drive it** — the flows to reach for, and the flags that matter when a command is
run by an agent rather than typed at a prompt.

> **The exhaustive reference is `gitnow skill`.** It prints every command, flag,
> config key, and workflow as one machine-readable document. Run it when you need
> a detail this file omits; treat it as the source of truth over anything here.

## Running gitnow from Claude Code

**Always pass `--no-shell`.** By default gitnow spawns an interactive sub-shell in
the directory it selected, which hangs a non-interactive caller. With `--no-shell`
it prints the path to stdout instead:

```bash
cd "$(gitnow api-bookings --no-shell)"
```

Two more things worth knowing before you script it:

- Every selection command is interactive when it can't resolve a single match. Pass
  enough query terms (or `--repos`) to make the match unambiguous, or the command
  will sit waiting for a picker nobody is driving.
- `--chooser-file <path>` writes the chosen path to a file and implies `--no-shell`.
  This is what the shell integration uses; prefer plain `--no-shell` in scripts.

## Cloning and jumping to a repo

The default subcommand searches the cache, clones the repo if it isn't local yet,
and enters it. Query terms are ANDed — every term must match:

```bash
gitnow api bookings --no-shell     # all terms must match; best match wins
gitnow understory-io/gitnow --no-shell
```

fzf-style operators work on any term. Quote the ones your shell would eat:

| Operator | Meaning |
|---|---|
| `'exact` | literal, no fuzzy matching |
| `^prefix` | anchored at the start |
| `suffix$` | anchored at the end |
| `!exclude` | must *not* match |

```bash
gitnow canopy '!ingest' --no-shell     # canopy repos, excluding the ingest ones
gitnow "'gitnow" '!github' --no-shell  # literal "gitnow", not on github.com
```

Terms are joined with a space and parsed as one fzf pattern, so quoting per term
and quoting the whole query are equivalent.

Useful modifiers:

| Flag | Use it when |
|---|---|
| `-i, --interactive` | you want the picker even with a query (pre-filtered by it) |
| `--no-clone` | you only want repos that already exist locally |
| `--force-refresh` | the local clone is broken and you want a fresh one |
| `--force-cache-update` | refresh the cache before searching |
| `--no-cache` | bypass the cache and hit the providers directly |

If a repo you expect is missing, the cache is stale — see [Cache](#cache).

### Batch cloning

`clone` takes a regex against the repo's relative path (`<host>/<org>/<name>`) and
clones every match, up to 5 concurrently, skipping what's already local:

```bash
gitnow clone --search 'understory-io/canopy-.*'
```

This one has no `--no-shell` — it never spawns a shell.

## Setting up a multi-repo workspace

This is the flow to use for any task touching more than one repo, and the one to
prefer over hand-rolled `git clone`s into a temp directory. A *project* is a
directory under `~/.gitnow/projects/<name>/` holding a full clone of each repo you
picked, optionally seeded from a template.

```bash
gitnow project create billing-refactor \
  --repos api-bookings --repos api-payment --repos canopy-models \
  --no-template --no-shell
```

That prints the workspace path and is fully non-interactive — which is the point.
Each `--repos` value is fuzzy-matched against the cache, so partial names are fine
as long as they're unambiguous. Leave `--repos` off and you get an interactive
multi-select picker instead; leave `--no-template` off and you get a template
prompt when templates exist.

| Flag | Description |
|---|---|
| `[NAME]` | project name; prompted for if omitted |
| `-r, --repos` | repo to include, fuzzy-matched. Repeatable |
| `-t, --template` | template to bootstrap from |
| `--no-template` | skip template selection entirely |
| `--no-cache` | skip the cache when listing repos |
| `--no-shell` | print the path instead of spawning a shell |

Templates live in `~/.gitnow/templates/`; each subdirectory is one template and
its contents are copied into the new project.

### The workspace-per-task pattern

Projects are cheap and disposable, which makes them the natural unit for *one
piece of work*: a task, a bug investigation, a spike, or an agent working through
something on its own. Instead of doing every job in the same long-lived clones
under `~/git`, give each job its own workspace holding exactly the repos it needs.

```bash
# One task, one workspace
gitnow project create billing-refactor --repos api-bookings --repos api-payment \
  --no-template --no-shell

# A second, unrelated task — entirely independent
gitnow project create investigate-webhook-lag --repos api-notifications \
  --no-template --no-shell
```

Each lands in its own directory under `~/.gitnow/projects/<name>/` with its own
full clones. Nothing is shared, so:

- **Parallel work doesn't collide.** Two tasks touching the same repo get two
  separate checkouts. Neither one's branch, uncommitted changes, or half-finished
  rebase is visible to the other, and neither can disturb your primary clone in
  `~/git`.
- **Every job starts clean.** A fresh workspace has no leftover state from the
  last thing you did — no stale branch checked out, no stray build artifacts, no
  local edits you'd forgotten about.
- **The repo set is scoped to the work.** The workspace contains the repos the
  task needs and nothing else, which keeps searching, building, and reviewing
  narrow.
- **Teardown is one command.** When the work lands, `gitnow project delete <name>`
  removes the whole thing; there's nothing to unpick. Age-based cleanup
  (`--older-than`) sweeps up whatever you forget.

Naming the workspace after the work — the ticket, the branch, the question being
answered — is what makes a directory of them readable later. `gitnow project list
--repos` shows every workspace currently in flight and what's in each.

Grow a workspace as the work turns out to need more than you expected, rather than
starting over:

```bash
gitnow project add billing-refactor --repos api-receipts
```

### Working with an existing project

```bash
gitnow project billing-refactor --no-shell   # print its path
gitnow project list --repos                  # every project and its repos
gitnow project list --json                   # the same, machine-readable
```

`project list --json` is the one to parse when you need to know what a workspace
contains.

### Changing and retiring a project

```bash
gitnow project add billing-refactor --repos api-receipts
gitnow project remove billing-refactor --repos canopy-models --force
gitnow project delete billing-refactor --force
```

`add`/`remove` take repeatable `--repos`; `remove` and `delete` prompt unless you
pass `-f, --force`. Bulk cleanup is by age:

```bash
gitnow project delete --older-than 30 --force --quiet
gitnow project delete --before 2026-01-01 --force
```

`--older-than` and `--before` conflict with a project name and with each other.
Matching projects are previewed before deletion unless `-q, --quiet` is set.
Retention can also run automatically — see `auto_delete_older_than_days` in
`gitnow skill`.

## Worktrees

Use a worktree when you need one repo on a second branch without disturbing the
existing checkout. gitnow bare-clones the repo, lists its remote branches, and
adds a worktree at `<project>/<sanitized-branch>/`:

```bash
gitnow worktree gitnow -b feature/fzf-queries --no-shell
```

`-b, --branch` skips the branch picker — pass it, or the command goes interactive.
Also takes `--no-cache` and `--no-shell`.

## Cache

gitnow searches a local cache of repos fetched from its configured providers,
stored at `~/.cache/gitnow` and valid for 7 days by default.

```bash
gitnow update    # refresh it; run this when a new repo isn't showing up
```

A missing repo is almost always a stale cache rather than a bad query, so reach
for `gitnow update` before assuming the repo isn't there. `--force-cache-update`
folds the refresh into a search, and `--no-cache` skips the cache for one command.

## Shell integration (humans, not agents)

```zsh
eval "$(gitnow init zsh)"
```

zsh only — there is no bash or fish subcommand. This defines three functions that
change the current shell's directory (via the chooser-file mechanism) instead of
spawning a sub-shell, and kicks off a background `gitnow update` on each call:

| Function | Behaviour |
|---|---|
| `git-now <query…>` | jump to the best match |
| `gn <query…>` | short alias for `git-now` |
| `gi <query…>` | jump interactively from a pre-filtered picker, like zoxide's `zi` |

Inside Understory, gitnow ships this integration as a forest component, so
`eval "$(forest shell zsh)"` loads it with no gitnow-specific line in your rc file
— see the `include: shell: init:` block in `forest.cue`.

## Configuration

Resolved in priority order: `-c, --config <PATH>`, then `$GITNOW_CONFIG`, then
`~/.config/gitnow/gitnow.toml`. At least one provider must be configured or
nothing will be found.

Defaults worth knowing:

| Path | What lives there |
|---|---|
| `~/git` | cloned repositories |
| `~/.cache/gitnow` | the repository cache |
| `~/.gitnow/projects` | multi-repo workspaces |
| `~/.gitnow/templates` | project templates |

`gitnow skill` documents the full TOML schema — providers, custom clone and
worktree commands, `post_clone_command` / `post_update_command` hooks, and cache
duration.

## Distribution

gitnow is published to the Forest registry as a `TOOL_EXTERNAL` component: this
repo's release pipeline builds the tarballs and hosts them, and `forest.cue`
records the URLs plus their hashes. Install it with:

```bash
forest global add understory/gitnow
```

`forest.cue` carries the full publish procedure in a comment, including the
`forest tool hash` invocation that produces the hashes. Bump the version there in
the same change as the release that produced the artifacts.
