use zsh::ZshShell;

pub mod zsh;

#[derive(clap::Parser)]
pub struct Shell {
    #[command(subcommand)]
    shell: ShellSubcommands,
}

impl Shell {
    pub async fn execute(&mut self) -> anyhow::Result<()> {
        self.shell.execute().await?;

        Ok(())
    }
}

#[derive(clap::Subcommand)]
pub enum ShellSubcommands {
    #[command()]
    Zsh(ZshShell),
}

impl ShellSubcommands {
    pub async fn execute(&mut self) -> anyhow::Result<()> {
        match self {
            ShellSubcommands::Zsh(zsh) => zsh.execute().await?,
        }

        Ok(())
    }
}
