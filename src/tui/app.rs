use std::error::Error;
use std::collections::HashSet;
use crate::beni_cli::{BeniCli, AppConfig, HtmlParser, DealType};
use crate::benifex::discount_response::Discount;
use ratatui::widgets::ListState;

#[derive(PartialEq)]
pub enum AppState {
    CookieInput,
    CategoryList,
    DiscountList,
    DiscountDetails,
}

pub struct App {
    pub state: AppState,
    pub cookies: String,
    pub categories: Vec<String>,
    pub discounts: Vec<(String, Discount, Option<DealType>)>, // (Category Name, Discount, DealType)
    pub category_list_state: ListState,
    pub discount_list_state: ListState,
    pub selected_discount_details: Option<crate::benifex::discount_view::DiscountView>,
    pub selected_discount_code: Option<String>,
    pub cli: Option<BeniCli>,
    pub config: AppConfig,
    pub parser: HtmlParser,
    pub clipboard: Option<arboard::Clipboard>,
    pub error_message: Option<String>,
    pub search_mode: bool,
    pub search_query: String,
}

impl App {
    pub fn new(config: AppConfig) -> App {
        let parser = HtmlParser::new(config.clone());
        App {
            state: AppState::CookieInput,
            cookies: String::new(),
            categories: Vec::new(),
            discounts: Vec::new(),
            category_list_state: ListState::default(),
            discount_list_state: ListState::default(),
            selected_discount_details: None,
            selected_discount_code: None,
            cli: None,
            config,
            parser,
            clipboard: arboard::Clipboard::new().ok(),
            error_message: None,
            search_mode: false,
            search_query: String::new(),
        }
    }

    pub fn get_current_discounts(&self) -> Vec<&(String, Discount, Option<DealType>)> {
        if self.categories.is_empty() {
            return Vec::new();
        }
        let active_cat_name = &self.categories[self.category_list_state.selected().unwrap_or(0)];
        let search_lower = self.search_query.to_lowercase();
        
        let filtered_by_cat = if active_cat_name == "All Discounts" {
            let mut seen = HashSet::new();
            let mut result = Vec::new();
            for d in &self.discounts {
                if seen.insert(d.1.id) {
                    result.push(d);
                }
            }
            result
        } else {
            self.discounts.iter().filter(|(c, _, _)| c == active_cat_name).collect::<Vec<_>>()
        };

        if self.search_query.is_empty() {
            filtered_by_cat
        } else {
            filtered_by_cat.into_iter().filter(|(cat, d, _)| {
                cat.to_lowercase().contains(&search_lower) || d.name.to_lowercase().contains(&search_lower)
            }).collect()
        }
    }

    pub async fn fetch_data(&mut self) -> Result<(), Box<dyn Error>> {
        let cli = BeniCli::new(self.config.clone(), self.cookies.clone());
        match cli.fetch_discounts().await {
            Ok(response) => {
                let mut all_discounts = Vec::new();
                let mut categories = Vec::new();
                categories.push("All Discounts".to_string());
                
                for category in response.function_data.result.discount_structure.categories {
                    categories.push(category.name.clone());
                    for discount in category.discounts {
                        let deal_type = self.parser.parse_discount_from_highlight(&discount.description_highlight);
                        all_discounts.push((category.name.clone(), discount, deal_type));
                    }
                }
                
                self.categories = categories;
                self.discounts = all_discounts;
                self.cli = Some(cli);
                self.state = AppState::CategoryList;
                if !self.categories.is_empty() {
                    self.category_list_state.select(Some(0));
                }
                Ok(())
            }
            Err(e) => {
                self.error_message = Some(format!("Error fetching discounts: {}", e));
                Err(e)
            }
        }
    }

    pub async fn fetch_details(&mut self) -> Result<(), Box<dyn Error>> {
        let current_discounts = self.get_current_discounts();
        if let Some(index) = self.discount_list_state.selected() {
            if index < current_discounts.len() {
                let discount_id = current_discounts[index].1.id;
                if let Some(cli) = &self.cli {
                    match cli.fetch_discount_item(discount_id).await {
                        Ok(details) => {
                            let html = &details.function_data.result.description_long_html;
                            self.selected_discount_code = None;
                            
                            if self.parser.has_discount_code(html) {
                                if let Ok(code) = cli.fetch_discount_code(discount_id).await {
                                    if !code.is_empty() {
                                        self.selected_discount_code = Some(code.trim_matches('"').to_string());
                                    }
                                }
                            }
                            
                            self.selected_discount_details = Some(details);
                            self.state = AppState::DiscountDetails;
                            Ok(())
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Error fetching details: {}", e));
                            Err(e)
                        }
                    }
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}