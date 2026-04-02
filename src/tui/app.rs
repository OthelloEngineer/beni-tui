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

#[derive(Clone, Debug, PartialEq)]
pub enum SearchState {
    None,
    Typing(String),
    Applied(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CategoryFilter {
    All,
    Specific(String),
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
    pub search_state: SearchState,
    pub category_filter: CategoryFilter,
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
            search_state: SearchState::None,
            category_filter: CategoryFilter::All,
        }
    }

    pub fn get_current_discounts(&self) -> Vec<usize> {
        let (search_active, search_lower) = match &self.search_state {
            SearchState::Typing(q) | SearchState::Applied(q) => (true, q.to_lowercase()),
            SearchState::None => (false, String::new()),
        };
        
        let filtered_by_cat = match &self.category_filter {
            CategoryFilter::All => {
                let mut seen = HashSet::new();
                let mut result = Vec::new();
                for (idx, d) in self.discounts.iter().enumerate() {
                    if seen.insert(d.1.id) {
                        result.push(idx);
                    }
                }
                result
            },
            CategoryFilter::Specific(cat_name) => {
                self.discounts
                    .iter()
                    .enumerate()
                    .filter(|(_, (c, _, _))| c == cat_name)
                    .map(|(idx, _)| idx)
                    .collect::<Vec<_>>()
            }
        };

        if !search_active || search_lower.is_empty() {
            filtered_by_cat
        } else {
            filtered_by_cat.into_iter().filter(|&idx| {
                let (cat, d, _) = &self.discounts[idx];
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
        let current_discounts_idx = self.get_current_discounts();
        
        let index = match self.discount_list_state.selected() {
            Some(i) => i,
            None => return Ok(()),
        };

        if index >= current_discounts_idx.len() {
            return Ok(());
        }

        let actual_idx = current_discounts_idx[index];
        let discount_id = self.discounts[actual_idx].1.id;
        
        let cli = match &self.cli {
            Some(c) => c,
            None => return Ok(()),
        };

        let details = match cli.fetch_discount_item(discount_id).await {
            Ok(d) => d,
            Err(e) => {
                self.error_message = Some(format!("Error fetching details: {}", e));
                return Err(e);
            }
        };

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
}