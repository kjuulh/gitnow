use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub providers: Providers,
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
    pub users: Vec<GitHubUser>,
    #[serde(default)]
    pub organisations: Vec<GitHubOrganisation>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubUser(String);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct GitHubOrganisation(String);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Gitea {
    pub url: String,
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
              [[providers.github]]  
              users = ["kjuulh"]
              organisations = ["lunarway"]

              [[providers.github]]  
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

        assert_eq!(
            Config {
                providers: Providers {
                    github: vec![
                        GitHub {
                            users: vec![GitHubUser("kjuulh".into())],
                            organisations: vec![GitHubOrganisation("lunarway".into())]
                        },
                        GitHub {
                            users: vec![GitHubUser("other".into())],
                            organisations: vec![GitHubOrganisation("org".into())]
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
                }
            },
            config
        );

        Ok(())
    }
}
