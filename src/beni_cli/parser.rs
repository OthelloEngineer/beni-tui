use regex::Regex;
use crate::beni_cli::config::AppConfig;
use tracing::debug;

#[derive(Debug, PartialEq, Clone)]
pub enum DealType {
    Percentage(i8),
    Trial(String),
}

#[derive(Clone)]
pub struct HtmlParser {
    config: AppConfig,
    re_link: Regex,
    re_code: Regex,
    re_paragraph: Regex,
    re_html_tags: Regex,
}

impl HtmlParser {
    pub fn new(config: AppConfig) -> Self {
        let re_link = Regex::new(&format!(r#"{}="([^"]+)""#, config.benifex.external_link_reference_attr)).unwrap();
        let re_code = Regex::new(&format!(r#"{}="([^"]+)""#, config.benifex.unique_discount_code_url_attr)).unwrap();
        let re_paragraph = Regex::new(&format!(r#"<p class="{}">([\s\S]*?)</p>"#, config.benifex.paragraph_node_class)).unwrap();
        let re_html_tags = Regex::new(r"<[^>]*>").unwrap();

        Self { config, re_link, re_code, re_paragraph, re_html_tags }
    }

    pub fn extract_link(&self, html: &str) -> Option<String> {
        let link = self.re_link.captures(html).map(|caps| {
            let mut url = caps.get(1).unwrap().as_str().to_string();
            url = url.replace("&amp;", "&");
            url
        });
        debug!("Extracted link: {:?}", link);
        link
    }

    pub fn has_discount_code(&self, html: &str) -> bool {
        let has_code = self.re_code.is_match(html);
        debug!("Has discount code: {}", has_code);
        has_code
    }

    pub fn extract_paragraphs(&self, html: &str) -> Vec<String> {
        let mut paragraphs = Vec::new();
        for cap in self.re_paragraph.captures_iter(html) {
            let p_html = cap.get(1).unwrap().as_str();
            let p_br = p_html.replace("<br>", "\n").replace("<br/>", "\n");
            let p_text = self.re_html_tags.replace_all(&p_br, "").trim().to_string();
            if !p_text.is_empty() {
                paragraphs.push(p_text);
            }
        }
        debug!("Extracted {} paragraphs", paragraphs.len());
        paragraphs
    }
    
    pub fn parse_discount_from_highlight(&self, highlight: &str) -> Option<DealType> {
        let percentage_regex = Regex::new(r"(\d+)\s*%").unwrap();
        if let Some(caps) = percentage_regex.captures(highlight) {
            let discount_str = caps.get(1)?.as_str();
            if let Ok(discount) = discount_str.parse::<i8>() {
                debug!("Parsed percentage discount: {}%", discount);
                return Some(DealType::Percentage(discount));
            }
        }
        
        let trial_regex = Regex::new(&self.config.benifex.trial_regex).unwrap();
        if let Some(caps) = trial_regex.captures(highlight) {
            let trial_str = caps.get(1)?.as_str();
            debug!("Parsed trial discount: {}", trial_str);
            return Some(DealType::Trial(trial_str.to_string()));
        }
        None
    }
}