use regex::Regex;
use serde_derive::{Serialize, Deserialize};

pub struct DiscountParser {
    pub trial_regex: Regex,
}

#[derive(Serialize, Deserialize)]
pub struct TrialSynonyms {
    synonyms: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum DealType {
    Percentage(i8),
    Trial(String),
}

impl DiscountParser {
    pub fn new(trial_regex: Regex) -> Self {
        Self { trial_regex }
    }

    pub fn parse_discount_from_highlight(&self, highlight: &str) -> Option<DealType> {
        let percentage_regex = Regex::new(r"(\d+)\s*%").unwrap();
        if let Some(caps) = percentage_regex.captures(highlight) {
            let discount_str = caps.get(1)?.as_str();
            let discount = discount_str.parse::<i8>().ok()?;
            return Some(DealType::Percentage(discount));
        }
        if let Some(caps) = self.trial_regex.captures(highlight) {
            let trial_str = caps.get(1)?.as_str();
            return Some(DealType::Trial(trial_str.to_string()));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discount_from_highlight() {
        let trial_regex = Regex::new(r"(\d+\s*(?:måneder|dage))\s*gratis").unwrap();
        let parser = DiscountParser::new(trial_regex);

        let h1 = "12% rabat";
        assert_eq!(parser.parse_discount_from_highlight(h1), Some(DealType::Percentage(12)));

        let h2 = "Spar 25% på alt!";
        assert_eq!(parser.parse_discount_from_highlight(h2), Some(DealType::Percentage(25)));

        let h3 = "2 måneder gratis";
        assert_eq!(parser.parse_discount_from_highlight(h3), Some(DealType::Trial("2 måneder".to_string())));

        let h4 = "45 dage gratis";
        assert_eq!(parser.parse_discount_from_highlight(h4), Some(DealType::Trial("45 dage".to_string())));

        let h5 = "Ingen rabat";
        assert_eq!(parser.parse_discount_from_highlight(h5), None);
    }
}