use crate::config::Config;

#[derive(Debug)]
pub struct App {
    pub config: Config,
}

impl App {
    pub async fn new_static(config: Config) -> anyhow::Result<&'static App> {
        Ok(Box::leak(Box::new(App { config })))
    }
}
