#[derive(clap::Parser)]
pub struct ZshShell {}

const SCRIPT: &str = include_str!("../../../include/shell/zsh.sh");

impl ZshShell {
    pub async fn execute(&mut self) -> anyhow::Result<()> {
        println!("{}", SCRIPT);

        Ok(())
    }
}
