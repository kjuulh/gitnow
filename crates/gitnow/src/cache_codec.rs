use std::io::Cursor;

use anyhow::Context;
use prost::Message;

use crate::{app::App, git_provider::Repository};

mod proto_codec {
    include!("gen/gitnow.v1.rs");
}

pub struct CacheCodec {}

impl CacheCodec {
    pub fn new() -> Self {
        Self {}
    }

    pub fn serialize_repositories(&self, repositories: &[Repository]) -> anyhow::Result<Vec<u8>> {
        let mut codec_repos = proto_codec::Repositories::default();

        for repo in repositories.iter().cloned() {
            codec_repos.repositories.push(proto_codec::Repository {
                provider: repo.provider,
                owner: repo.owner,
                repo_name: repo.repo_name,
                ssh_url: repo.ssh_url,
            });
        }

        Ok(codec_repos.encode_to_vec())
    }

    pub fn deserialize_repositories(&self, content: Vec<u8>) -> anyhow::Result<Vec<Repository>> {
        let codex_repos = proto_codec::Repositories::decode(&mut Cursor::new(content))
            .context("failed to decode protobuf repositories")?;

        let mut repos = Vec::new();

        for codec_repo in codex_repos.repositories {
            repos.push(Repository {
                provider: codec_repo.provider,
                owner: codec_repo.owner,
                repo_name: codec_repo.repo_name,
                ssh_url: codec_repo.ssh_url,
            });
        }

        Ok(repos)
    }
}

pub trait CacheCodecApp {
    fn cache_codec(&self) -> CacheCodec;
}

impl CacheCodecApp for &'static App {
    fn cache_codec(&self) -> CacheCodec {
        CacheCodec::new()
    }
}
