use std::path::{Path, PathBuf};

/// Manages an optional chooser file that the shell wrapper reads after gitnow
/// exits.  When active, the selected directory path is written to the file
/// instead of being printed to stdout.
#[derive(Debug, Default)]
pub struct Chooser {
    path: Option<PathBuf>,
}

impl Chooser {
    pub fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    /// Returns `true` when a chooser file has been configured.
    pub fn is_active(&self) -> bool {
        self.path.is_some()
    }

    /// Write `dir` to the chooser file.  If no chooser file is configured the
    /// path is printed to stdout (preserving the old `--no-shell` behaviour).
    pub fn set(&self, dir: &Path) -> anyhow::Result<()> {
        match &self.path {
            Some(chooser_path) => {
                std::fs::write(chooser_path, dir.display().to_string())?;
            }
            None => {
                println!("{}", dir.display());
            }
        }
        Ok(())
    }
}
