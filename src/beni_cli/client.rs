use crate::benifex;
use crate::beni_cli::config::AppConfig;
use tracing::{error, info};

pub struct BeniCli {
    pub config: AppConfig,
    pub cookies: String,
}

impl BeniCli {
    pub fn new(config: AppConfig, cookies: String) -> Self {
        Self { config, cookies }
    }

    pub async fn fetch_discounts(&self) -> Result<benifex::discount_response::FetchStructureResponse, Box<dyn std::error::Error>> {
        let url = format!("{}{}", self.config.benifex.base_url, self.config.benifex.discount_path);
        info!("Fetching all discounts from {}", url);
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header("Cookie", self.cookies.clone())
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to {}: {}", url, e);
                e
            })?;
        let body = response.text().await.map_err(|e| {
            error!("Failed to get response text from {}: {}", url, e);
            e
        })?;
        let discount_response = serde_json::from_str::<benifex::discount_response::FetchStructureResponse>(&body).map_err(|e| {
            error!("Failed to parse discounts JSON: {}. Body: {}. Request: {}, cookies: {}", e, body, url, self.cookies);
            e
        })?;
        Ok(discount_response)
    }

    pub async fn fetch_discount_item(&self, discount_id: i64) -> Result<benifex::discount_view::DiscountView, Box<dyn std::error::Error>> {
        let url = format!("{}{}{}", self.config.benifex.base_url, self.config.benifex.discount_item, discount_id);
        info!("Fetching discount item {} from {}", discount_id, url);
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header("Cookie", self.cookies.clone())
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request for discount item {}: {}", discount_id, e);
                e
            })?;
        let body = response.text().await.map_err(|e| {
            error!("Failed to get response text for discount item {}: {}", discount_id, e);
            e
        })?;
        let discount_view = serde_json::from_str::<benifex::discount_view::DiscountView>(&body).map_err(|e| {
            error!("Failed to parse discount item {} JSON: {}. Body: {}. Request: {}", discount_id, e, body, url);
            e
        })?;
        Ok(discount_view)
    }

    pub async fn fetch_discount_code(&self, discount_id: i64) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}{}", self.config.benifex.base_url, self.config.benifex.discount_code_path, discount_id);
        info!("Fetching discount code for {} from {}", discount_id, url);
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header("Cookie", self.cookies.clone())
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request for discount code {}: {}", discount_id, e);
                e
            })?;
        let body = response.text().await.map_err(|e| {
            error!("Failed to get response text for discount code {}: {}", discount_id, e);
            e
        })?;
        let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            error!("Failed to parse discount code {} JSON: {}. Body: {}. Request: {}", discount_id, e, body, url);
            e
        })?;
        
        if let Some(code) = json.get("functionData")
            .and_then(|fd| fd.get("result"))
            .and_then(|r| r.as_str()) {
            Ok(code.to_string())
        } else {
            error!("Discount code not found in response for {}. Body: {}. Request: {}", discount_id, body, url );
            Ok("".to_string())
        }
    }
}