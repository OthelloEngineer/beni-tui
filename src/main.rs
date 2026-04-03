pub mod beni_cli;
pub mod benifex;
pub mod tui;
pub mod logging;

use beni_cli::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = logging::init();
    let config = AppConfig::load("config.yaml")?;
    tui::run_tui(config).await?;
    Ok(())
}