//! Zero-config first run.
//!
//! A fresh install has no config file at all.  [`crate::config::Config::from_file`]
//! creates an empty one, which parses into a `Config` with no providers, and
//! without providers there is nothing to search or clone — so a first run used
//! to do nothing until the user hand-wrote a config.
//!
//! When — and only when — *no* providers are configured, gitnow sniffs the
//! environment for GitHub credentials and synthesises a provider for
//! github.com pointing at the user's own account.  The provider is layered on
//! at runtime and never written to disk, which means:
//!
//! * the token is not persisted, and is re-resolved on every run, and
//! * a config the user writes later simply takes over, with no migration.
//!
//! The trigger is [`crate::config::Providers::is_empty`] — *providers*, not
//! settings.  Customising `[settings]` does not turn the default off, and
//! configuring any provider (GitHub *or* Gitea) does.

use std::process::Stdio;
use std::time::Duration;

use crate::config::{GitHub, GitHubAccessToken, GitHubUser};

/// Environment variables consulted for a token, in precedence order, after
/// `gh auth token` has come up empty.
const TOKEN_ENV_VARS: [&str; 2] = ["GH_TOKEN", "GITHUB_TOKEN"];

/// Upper bound on a single `gh` invocation, so a hanging or wedged `gh` cannot
/// wedge gitnow along with it.
const GH_TIMEOUT: Duration = Duration::from_secs(10);

/// Stand-in for `current_user` when a token was found but the login could not
/// be resolved (no `gh` on PATH).  gitnow only checks whether `current_user` is
/// set; the value is never sent anywhere, as `/user/repos` is scoped by the
/// token itself.
const UNKNOWN_LOGIN: &str = "authenticated-user";

/// Shown once, on stderr, when there is no config *and* nothing to sniff.
/// A hint rather than an error: gitnow must never prompt or block automation
/// (Claude Code and other callers pass `--no-shell`).
pub const NO_AUTH_HINT: &str = "\
gitnow: no providers configured and no GitHub auth found.
  - run `gh auth login` — gitnow then uses your own GitHub automatically, or
  - set GH_TOKEN or GITHUB_TOKEN in your environment, or
  - write ~/.config/gitnow/gitnow.toml (see `gitnow skill`, or
    https://github.com/understory-io/gitnow#configuration)";

/// A snapshot of everything the sniff read from the environment.
///
/// Split out from the IO that gathers it so the precedence rules and the shape
/// of the synthesised provider can be tested without a `gh` binary.
#[derive(Default, PartialEq, Clone)]
pub struct Environment {
    /// Token reported by `gh auth token`, when `gh` is installed and authenticated.
    pub gh_token: Option<String>,

    /// Login reported by `gh api user`, when it could be resolved.
    pub gh_login: Option<String>,

    /// *Names* of the token environment variables that are set, in
    /// [`TOKEN_ENV_VARS`] precedence order.  Only the names are kept: the value
    /// is referenced indirectly via [`GitHubAccessToken::Env`] and resolved by
    /// the provider at request time.
    pub token_env_vars: Vec<String>,
}

/// Redacted — a token must never reach a log line or `--verbose` output.
impl std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Environment")
            .field("gh_token", &self.gh_token.as_ref().map(|_| Redacted))
            .field("gh_login", &self.gh_login)
            .field("token_env_vars", &self.token_env_vars)
            .finish()
    }
}

pub(crate) struct Redacted;

impl std::fmt::Debug for Redacted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<redacted>")
    }
}

impl Environment {
    /// The token to authenticate with: `gh auth token`, then `GH_TOKEN`, then
    /// `GITHUB_TOKEN`, then nothing.
    fn access_token(&self) -> Option<GitHubAccessToken> {
        if let Some(token) = &self.gh_token {
            return Some(GitHubAccessToken::Direct(token.clone()));
        }

        self.token_env_vars
            .first()
            .map(|env| GitHubAccessToken::Env { env: env.clone() })
    }
}

/// Synthesise the default github.com provider for `env`, or `None` when no
/// GitHub credentials could be found at all.
///
/// The shape matches what a user would hand-write for their own account:
/// `url = None` (github.com), `current_user` set so `/user/repos` is fetched,
/// and `users = [login]`.  Organisations are deliberately left empty —
/// `/user/repos?type=all` already covers the org repositories the token can
/// see, so seeding them would only cost extra requests.
pub fn default_github_provider(env: &Environment) -> Option<GitHub> {
    let access_token = env.access_token()?;
    let login = env.gh_login.clone();

    Some(GitHub {
        url: None,
        access_token,
        current_user: Some(login.clone().unwrap_or_else(|| UNKNOWN_LOGIN.to_string())),
        users: login
            .map(|login| vec![GitHubUser::from(login)])
            .unwrap_or_default(),
        organisations: Vec::new(),
    })
}

/// Read the environment: `gh auth token`, the token environment variables, and
/// — only when there is a token worth using — the login from `gh api user`.
pub async fn probe_environment() -> Environment {
    let gh_token = gh(&["auth", "token"]).await;
    let token_env_vars = present_token_env_vars(|key| std::env::var(key).ok());

    // `gh api user` is a network round-trip; skip it when there is nothing to
    // authenticate with anyway.
    let gh_login = if gh_token.is_some() || !token_env_vars.is_empty() {
        gh(&["api", "user", "--jq", ".login"]).await
    } else {
        None
    };

    Environment {
        gh_token,
        gh_login,
        token_env_vars,
    }
}

/// The token environment variables that are set to a non-empty value, in
/// precedence order.  Takes the lookup as an argument so it can be tested
/// without mutating the process environment.
fn present_token_env_vars(lookup: impl Fn(&str) -> Option<String>) -> Vec<String> {
    TOKEN_ENV_VARS
        .iter()
        .filter(|key| lookup(key).is_some_and(|value| !value.trim().is_empty()))
        .map(|key| (*key).to_string())
        .collect()
}

/// Run `gh` and return its trimmed stdout, or `None` for any reason it did not
/// produce output: not installed, not authenticated, timed out, or failed.
///
/// stdin and stderr are closed so `gh` can never prompt and never writes to the
/// user's terminal.  Neither stdout nor stderr is ever logged — stdout holds
/// the token.
async fn gh(args: &[&str]) -> Option<String> {
    let command = tokio::process::Command::new("gh")
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        // So a `gh` that outlives the timeout below is reaped, not orphaned.
        .kill_on_drop(true)
        .output();

    match tokio::time::timeout(GH_TIMEOUT, command).await {
        Err(_) => {
            tracing::debug!(args = ?args, "gh timed out, skipping");
            None
        }
        Ok(Err(e)) => {
            tracing::debug!(error = %e, args = ?args, "could not run gh, skipping");
            None
        }
        Ok(Ok(output)) if !output.status.success() => {
            tracing::debug!(
                status = output.status.code(),
                args = ?args,
                "gh exited unsuccessfully, skipping"
            );
            None
        }
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (!stdout.is_empty()).then_some(stdout)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::{Config, Gitea, Providers};

    fn env(gh_token: Option<&str>, gh_login: Option<&str>, token_env_vars: &[&str]) -> Environment {
        Environment {
            gh_token: gh_token.map(str::to_string),
            gh_login: gh_login.map(str::to_string),
            token_env_vars: token_env_vars.iter().map(|v| (*v).to_string()).collect(),
        }
    }

    #[test]
    fn empty_providers_are_the_trigger() {
        assert!(Providers::default().is_empty());
        assert!(Config::from_string("").unwrap().providers.is_empty());
    }

    #[test]
    fn any_configured_provider_disables_the_trigger() {
        let github = Providers {
            github: vec![default_github_provider(&env(Some("token"), None, &[])).unwrap()],
            gitea: vec![],
        };
        assert!(!github.is_empty());

        let gitea = Providers {
            github: vec![],
            gitea: vec![Gitea {
                url: "https://gitea.example.com/api/v1".into(),
                access_token: None,
                current_user: None,
                users: vec![],
                organisations: vec![],
            }],
        };
        assert!(!gitea.is_empty());
    }

    #[test]
    fn customised_settings_alone_still_trigger() {
        let config = Config::from_string(
            r#"
              [settings]
              projects = { directory = "git" }
              clone_command = "jj git clone {{ ssh_url }} {{ path }}"
            "#,
        )
        .unwrap();

        assert!(config.providers.is_empty());
    }

    #[test]
    fn the_gh_cli_token_wins_over_the_environment() {
        let provider = default_github_provider(&env(
            Some("gh-cli-token"),
            None,
            &["GH_TOKEN", "GITHUB_TOKEN"],
        ))
        .unwrap();

        assert_eq!(
            provider.access_token,
            GitHubAccessToken::Direct("gh-cli-token".into())
        );
    }

    #[test]
    fn gh_token_wins_over_github_token() {
        let provider =
            default_github_provider(&env(None, None, &["GH_TOKEN", "GITHUB_TOKEN"])).unwrap();

        assert_eq!(
            provider.access_token,
            GitHubAccessToken::Env {
                env: "GH_TOKEN".into()
            }
        );
    }

    #[test]
    fn github_token_is_the_last_resort() {
        let provider = default_github_provider(&env(None, None, &["GITHUB_TOKEN"])).unwrap();

        assert_eq!(
            provider.access_token,
            GitHubAccessToken::Env {
                env: "GITHUB_TOKEN".into()
            }
        );
    }

    #[test]
    fn without_credentials_nothing_is_synthesised() {
        assert_eq!(
            default_github_provider(&env(None, Some("kjuulh"), &[])),
            None
        );
        assert_eq!(default_github_provider(&Environment::default()), None);
    }

    #[test]
    fn the_synthesised_provider_targets_your_own_github() {
        let provider =
            default_github_provider(&env(Some("gh-cli-token"), Some("kjuulh"), &[])).unwrap();

        assert_eq!(provider.url, None, "github.com, not an enterprise host");
        assert_eq!(provider.current_user, Some("kjuulh".into()));
        assert_eq!(provider.users, vec![GitHubUser::from("kjuulh".to_string())]);
        assert_eq!(provider.organisations, vec![]);
    }

    #[test]
    fn without_a_login_your_own_repos_are_still_listed() {
        let provider = default_github_provider(&env(Some("gh-cli-token"), None, &[])).unwrap();

        // `current_user` drives `/user/repos`, which the token alone scopes.
        assert!(provider.current_user.is_some());
        assert_eq!(provider.users, vec![]);
    }

    #[test]
    fn only_non_empty_token_variables_count() {
        let present = present_token_env_vars(|key| match key {
            "GH_TOKEN" => Some("   ".to_string()),
            "GITHUB_TOKEN" => Some("token".to_string()),
            _ => None,
        });

        assert_eq!(present, vec!["GITHUB_TOKEN".to_string()]);
        assert_eq!(present_token_env_vars(|_| None), Vec::<String>::new());
    }

    #[test]
    fn environment_tokens_are_referenced_by_name_never_by_value() {
        let provider = default_github_provider(&env(None, None, &["GH_TOKEN"])).unwrap();

        // The value is resolved by the provider at request time, so it never
        // even enters the synthesised config.
        assert_eq!(
            provider.access_token,
            GitHubAccessToken::Env {
                env: "GH_TOKEN".into()
            }
        );
    }

    #[test]
    fn debug_output_redacts_tokens() {
        let environment = env(Some("super-secret"), Some("kjuulh"), &["GH_TOKEN"]);
        let provider = default_github_provider(&environment).unwrap();

        for rendered in [format!("{environment:?}"), format!("{provider:?}")] {
            assert!(
                !rendered.contains("super-secret"),
                "token leaked into debug output: {rendered}"
            );
            assert!(
                rendered.contains("<redacted>"),
                "expected a redaction marker in: {rendered}"
            );
        }
    }

    #[tokio::test]
    async fn a_sniffed_token_is_never_written_to_the_config_file() -> anyhow::Result<()> {
        let path = std::env::temp_dir()
            .join(format!("gitnow-zero-config-{}", uuid::Uuid::new_v4()))
            .join("gitnow.toml");

        // A fresh install: the file does not exist yet.
        let config = Config::from_file(&path).await?;
        assert!(config.providers.is_empty());

        let provider = default_github_provider(&env(Some("super-secret"), Some("kjuulh"), &[]))
            .expect("a provider should be synthesised");

        // Synthesising and injecting the provider must not touch the file.
        let mut config = config;
        config.providers.github.push(provider);

        let on_disk = tokio::fs::read_to_string(&path).await?;
        assert_eq!(on_disk, "", "the config file must stay untouched");
        assert!(!on_disk.contains("super-secret"));

        tokio::fs::remove_dir_all(path.parent().unwrap()).await?;

        Ok(())
    }

    #[tokio::test]
    async fn probing_a_bare_environment_finds_nothing_and_does_not_panic() {
        // `gh` may or may not exist on the machine running the tests; either
        // way the probe must return, not hang or panic.
        let environment = probe_environment().await;

        let _ = default_github_provider(&environment);
    }
}
