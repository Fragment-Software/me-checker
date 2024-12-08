mod checker;
mod linker;
mod processor;

use crate::config::Config;

use checker::checker;
use dialoguer::{theme::ColorfulTheme, Select};
use linker::linker;

pub async fn menu() -> eyre::Result<()> {
    let config = Config::read_default().await;

    loop {
        let options = vec!["Check allocation", "Link wallets for claim", "Exit"];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choice:")
            .items(&options)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => checker(&config).await?,
            1 => linker(&config).await?,
            2 => {
                return Ok(());
            }
            _ => tracing::error!("Invalid selection"),
        }
    }
}
