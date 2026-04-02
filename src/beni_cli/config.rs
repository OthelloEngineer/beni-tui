use serde_derive::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(rename = "trialSynonyms")]
    pub trial_synonyms: Vec<String>,
    pub benifex: BenifexConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BenifexConfig {
    pub base_url: String,
    pub discount_path: String,
    pub discount_item: String,
    pub discount_code_path: String,
    pub paragraph_node_class: String,
    pub external_link_reference_attr: String,
    pub unique_discount_code_url_attr: String,
    pub trial_regex: String,
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        let config: AppConfig = serde_yaml::from_reader(file)?;
        Ok(config)
    }
}