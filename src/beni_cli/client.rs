use crate::benifex;
use crate::beni_cli::config::AppConfig;

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
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header("Cookie", self.cookies.clone())
            .send()
            .await?;
        let body = response.text().await?;
        let discount_response = serde_json::from_str::<benifex::discount_response::FetchStructureResponse>(&body)?;
        Ok(discount_response)
    }

    pub async fn fetch_discount_item(&self, discount_id: i64) -> Result<benifex::discount_view::DiscountView, Box<dyn std::error::Error>> {
        let url = format!("{}{}{}", self.config.benifex.base_url, self.config.benifex.discount_item, discount_id);
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header("Cookie", self.cookies.clone())
            .send()
            .await?;
        let body = response.text().await?;
        let discount_view = serde_json::from_str::<benifex::discount_view::DiscountView>(&body)?;
        Ok(discount_view)
    }

    pub async fn fetch_discount_code(&self, discount_id: i64) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}{}", self.config.benifex.base_url, self.config.benifex.discount_code_path, discount_id);
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header("Cookie", self.cookies.clone())
            .send()
            .await?;
        let body = response.text().await?;
        let json: serde_json::Value = serde_json::from_str(&body)?;
        
        if let Some(code) = json.get("functionData")
            .and_then(|fd| fd.get("result"))
            .and_then(|r| r.as_str()) {
            Ok(code.to_string())
        } else {
            Ok("".to_string())
        }
    }
}