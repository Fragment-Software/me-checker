use serde::Deserialize;
use std::path::Path;

const CONFIG_FILE_PATH: &str = "data/config.toml";

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Config {
    pub batch_size: usize,
    pub parallelism: usize,
}

impl Config {
    async fn read_from_file(path: impl AsRef<Path>) -> eyre::Result<Self> {
        let cfg_str = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&cfg_str)?)
    }

    pub async fn read_default() -> Self {
        Self::read_from_file(CONFIG_FILE_PATH)
            .await
            .expect("Default config to be valid")
    }
}
