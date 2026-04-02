pub mod beni_cli;
pub mod benifex;
pub mod tui;

use beni_cli::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load("config.yaml")?;
    tui::run_tui(config).await?;
    Ok(())
}