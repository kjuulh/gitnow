use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,

    #[serde(default)]
    pub providers: Providers,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    #[serde(default)]
    pub cache: Cache,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Cache {
    #[serde(default)]
    pub location: CacheLocation,

    #[serde(default)]
    pub duration: CacheDuration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheLocation(PathBuf);

impl From<PathBuf> for CacheLocation {
    fn from(value: PathBuf) -> Self {
        Self(value)
    }
}

impl From<CacheLocation> for PathBuf {
    fn from(value: CacheLocation) -> Self {
        value.0
    }
}

impl Default for CacheLocation {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();

        Self(home.join(".cache/gitnow"))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CacheDuration {
    Enabled(bool),
    Precise {
        #[serde(default)]
        days: u64,
        #[serde(default)]
        hours: u64,
        #[serde(default)]
        minutes: u64,
    },
}

impl CacheDuration {
    pub fn get_duration(&self) -> Option<std::time::Duration> {
        match self {
            CacheDuration::Enabled(true) => CacheDuration::default().get_duration(),
            CacheDuration::Enabled(false) => None,
            CacheDuration::Precise {
                days,
                hours,
                minutes,
            } => Some(
                std::time::Duration::from_days(*days)
                    + std::time::Duration::from_hours(*hours)
                    + std::time::Duration::from_mins(*minutes),
            ),
        }
    }
}

impl Default for CacheDuration {
    fn default() -> Self {
        Self::Precise {
            days: 1,
            hours: 0,
            minutes: 0,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Providers {
    #[serde(default)]
    pub github: Vec<GitHub>,
    #[serde(default)]
    pub gitea: Vec<Gitea>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHub {
    #[serde(default)]
    pub url: Option<String>,

    pub access_token: GitHubAccessToken,

    #[serde(default)]
    pub current_user: Option<String>,

    #[serde(default)]
    pub users: Vec<GitHubUser>,
    #[serde(default)]
    pub organisations: Vec<GitHubOrganisation>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubUser(String);

impl From<GitHubUser> for String {
    fn from(value: GitHubUser) -> Self {
        value.0
    }
}

impl<'a> From<&'a GitHubUser> for &'a str {
    fn from(value: &'a GitHubUser) -> Self {
        value.0.as_str()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubOrganisation(String);

impl From<GitHubOrganisation> for String {
    fn from(value: GitHubOrganisation) -> Self {
        value.0
    }
}

impl<'a> From<&'a GitHubOrganisation> for &'a str {
    fn from(value: &'a GitHubOrganisation) -> Self {
        value.0.as_str()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Gitea {
    pub url: String,

    #[serde(default)]
    pub access_token: Option<GiteaAccessToken>,

    #[serde(default)]
    pub current_user: Option<String>,

    #[serde(default)]
    pub users: Vec<GiteaUser>,
    #[serde(default)]
    pub organisations: Vec<GiteaOrganisation>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GiteaAccessToken {
    Direct(String),
    Env { env: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GitHubAccessToken {
    Direct(String),
    Env { env: String },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GiteaUser(String);

impl From<GiteaUser> for String {
    fn from(value: GiteaUser) -> Self {
        value.0
    }
}

impl<'a> From<&'a GiteaUser> for &'a str {
    fn from(value: &'a GiteaUser) -> Self {
        value.0.as_str()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GiteaOrganisation(String);
impl From<GiteaOrganisation> for String {
    fn from(value: GiteaOrganisation) -> Self {
        value.0
    }
}

impl<'a> From<&'a GiteaOrganisation> for &'a str {
    fn from(value: &'a GiteaOrganisation) -> Self {
        value.0.as_str()
    }
}

impl Config {
    pub async fn from_file(file_path: &Path) -> anyhow::Result<Config> {
        if !file_path.exists() {
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::File::create(file_path).await?;
        }

        let file_content = tokio::fs::read_to_string(file_path).await?;

        Self::from_string(&file_content)
    }

    pub fn from_string(content: &str) -> anyhow::Result<Config> {
        toml::from_str(content).context("failed to deserialize config file")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_can_parse_config() -> anyhow::Result<()> {
        let content = r#"
              [settings.cache]
              location = ".cache/gitnow"
              duration = { days = 2 }

              [[providers.github]]  
              current_user = "kjuulh"
              access_token = "some-token"
              users = ["kjuulh"]
              organisations = ["lunarway"]

              [[providers.github]]  
              access_token = { env = "something" }
              users = ["other"]
              organisations = ["org"]

              [[providers.gitea]]  
              url = "https://git.front.kjuulh.io/api/v1"
              current_user = "kjuulh"
              users = ["kjuulh"]
              organisations = ["lunarway"]

              [[providers.gitea]]  
              url = "https://git.front.kjuulh.io/api/v1"
              users = ["other"]
              organisations = ["org"]

              [[providers.gitea]]  
              url = "https://git.front.kjuulh.io/api/v1"
            "#;

        let config = Config::from_string(content)?;

        pretty_assertions::assert_eq!(
            Config {
                providers: Providers {
                    github: vec![
                        GitHub {
                            users: vec![GitHubUser("kjuulh".into())],
                            organisations: vec![GitHubOrganisation("lunarway".into())],
                            url: None,
                            access_token: GitHubAccessToken::Direct("some-token".into()),
                            current_user: Some("kjuulh".into())
                        },
                        GitHub {
                            users: vec![GitHubUser("other".into())],
                            organisations: vec![GitHubOrganisation("org".into())],
                            url: None,
                            access_token: GitHubAccessToken::Env {
                                env: "something".into()
                            },
                            current_user: None
                        }
                    ],
                    gitea: vec![
                        Gitea {
                            url: "https://git.front.kjuulh.io/api/v1".into(),
                            users: vec![GiteaUser("kjuulh".into())],
                            organisations: vec![GiteaOrganisation("lunarway".into())],
                            access_token: None,
                            current_user: Some("kjuulh".into())
                        },
                        Gitea {
                            url: "https://git.front.kjuulh.io/api/v1".into(),
                            users: vec![GiteaUser("other".into())],
                            organisations: vec![GiteaOrganisation("org".into())],
                            access_token: None,
                            current_user: None
                        },
                        Gitea {
                            url: "https://git.front.kjuulh.io/api/v1".into(),
                            users: vec![],
                            organisations: vec![],
                            access_token: None,
                            current_user: None
                        },
                    ]
                },
                settings: Settings {
                    cache: Cache {
                        location: PathBuf::from(".cache/gitnow").into(),
                        duration: CacheDuration::Precise {
                            days: 2,
                            hours: 0,
                            minutes: 0
                        }
                    }
                }
            },
            config
        );

        Ok(())
    }

    #[test]
    fn test_can_parse_empty_config() -> anyhow::Result<()> {
        let content = r#"
                # empty file
            "#;

        let config = Config::from_string(content)?;

        assert_eq!(
            Config {
                providers: Providers {
                    github: vec![],
                    gitea: vec![]
                },
                settings: Settings {
                    cache: Cache::default()
                }
            },
            config
        );

        Ok(())
    }
}
