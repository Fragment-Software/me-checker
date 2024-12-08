mod config;
mod crypto;
mod me;
mod utils;

use modules::menu;
use utils::logger::init_default_logger;

mod modules;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = init_default_logger();

    if let Err(e) = menu().await {
        tracing::error!("Execution stopped with an unexpected error: {e}");
    }

    Ok(())
}
