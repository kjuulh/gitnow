use tokio::sync::OnceCell;

use crate::{
    config::{Config, GitHub},
    zero_config,
};

#[derive(Debug)]
pub struct App {
    pub config: Config,

    /// The zero-config default provider, sniffed at most once per process and
    /// only when the config declares no providers at all.  Empty when nothing
    /// could be sniffed.
    zero_config_github: OnceCell<Vec<GitHub>>,
}

impl App {
    pub async fn new_static(config: Config) -> anyhow::Result<&'static App> {
        Ok(Box::leak(Box::new(App {
            config,
            zero_config_github: OnceCell::new(),
        })))
    }

    /// The GitHub providers to fetch repositories from.
    ///
    /// Normally the configured ones, returned untouched.  Only when *no*
    /// providers are configured at all does this fall back to a provider
    /// sniffed from the environment (see [`crate::zero_config`]), so that a
    /// fresh install works without a config file.  The sniff runs lazily — a
    /// configured install never shells out to `gh`, and neither do commands
    /// that don't list repositories — and at most once per process.
    pub async fn github_providers(&self) -> &[GitHub] {
        if !self.config.providers.is_empty() {
            return &self.config.providers.github;
        }

        self.zero_config_github
            .get_or_init(|| async {
                let environment = zero_config::probe_environment().await;

                match zero_config::default_github_provider(&environment) {
                    Some(provider) => {
                        tracing::debug!(
                            current_user = provider.current_user.as_deref(),
                            "no providers configured, using the default github.com provider"
                        );

                        vec![provider]
                    }
                    None => {
                        eprintln!("{}", zero_config::NO_AUTH_HINT);

                        Vec::new()
                    }
                }
            })
            .await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn configured_providers_are_used_as_is() -> anyhow::Result<()> {
        let config = Config::from_string(
            r#"
              [[providers.github]]
              current_user = "kjuulh"
              access_token = "some-token"
              users = ["kjuulh"]
              organisations = ["lunarway"]
            "#,
        )?;

        let app = App::new_static(config).await?;

        // Identical to the configured providers: no injected default, and no
        // `gh` shell-out on this path.
        assert_eq!(
            app.github_providers().await,
            app.config.providers.github.as_slice()
        );
        assert_eq!(app.github_providers().await.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn a_configured_gitea_provider_also_suppresses_the_default() -> anyhow::Result<()> {
        let config = Config::from_string(
            r#"
              [[providers.gitea]]
              url = "https://git.front.kjuulh.io/api/v1"
              current_user = "kjuulh"
            "#,
        )?;

        let app = App::new_static(config).await?;

        assert_eq!(app.github_providers().await, &[] as &[GitHub]);

        Ok(())
    }
}
