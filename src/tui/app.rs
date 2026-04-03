use std::error::Error;
use std::collections::HashSet;
use crate::beni_cli::{BeniCli, AppConfig, HtmlParser, DealType};
use crate::benifex::discount_response::Discount;
use ratatui::widgets::{ListState, TableState};
use tracing::{error, info};

#[derive(PartialEq, Clone, Copy)]
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

pub struct CategoryList {
    pub items: Vec<String>,
    pub state: ListState,
}

impl CategoryList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: ListState::default(),
        }
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        if !self.items.is_empty() {
            self.state.select(Some(0));
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i >= self.items.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i == 0 { self.items.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn selected_name(&self) -> Option<String> {
        self.state.selected().map(|i| self.items[i].clone())
    }
}

pub struct DiscountList {
    pub state: TableState,
    pub filtered_indices: Vec<usize>,
}

impl DiscountList {
    pub fn new() -> Self {
        Self {
            state: TableState::default(),
            filtered_indices: Vec::new(),
        }
    }

    pub fn update_indices(&mut self, indices: Vec<usize>) {
        self.filtered_indices = indices;
        if self.filtered_indices.is_empty() {
            self.state.select(None);
        } else if self.state.selected().is_none() {
            self.state.select(Some(0));
        } else if let Some(i) = self.state.selected() {
            if i >= self.filtered_indices.len() {
                self.state.select(Some(self.filtered_indices.len().saturating_sub(1)));
            }
        }
    }

    pub fn next(&mut self) {
        if self.filtered_indices.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i >= self.filtered_indices.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_indices.is_empty() { return; }
        let i = match self.state.selected() {
            Some(i) => if i == 0 { self.filtered_indices.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.state.select(Some(i));
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SortColumn {
    Category,
    Name,
    Deal,
    StartDate,
    EndDate,
}

impl SortColumn {
    pub fn next(self) -> Self {
        match self {
            SortColumn::Category => SortColumn::Name,
            SortColumn::Name => SortColumn::Deal,
            SortColumn::Deal => SortColumn::StartDate,
            SortColumn::StartDate => SortColumn::EndDate,
            SortColumn::EndDate => SortColumn::Category,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            SortColumn::Category => SortColumn::EndDate,
            SortColumn::Name => SortColumn::Category,
            SortColumn::Deal => SortColumn::Name,
            SortColumn::StartDate => SortColumn::Deal,
            SortColumn::EndDate => SortColumn::StartDate,
        }
    }
}

pub struct App {
    pub state: AppState,
    pub cookies: String,
    pub discounts: Vec<(String, Discount, Option<DealType>)>,
    pub category_list: CategoryList,
    pub discount_list: DiscountList,
    pub selected_discount_details: Option<crate::benifex::discount_view::DiscountView>,
    pub selected_discount_code: Option<String>,
    pub cli: Option<BeniCli>,
    pub config: AppConfig,
    pub parser: HtmlParser,
    pub clipboard: Option<arboard::Clipboard>,
    pub error_message: Option<String>,
    pub search_state: SearchState,
    pub category_filter: CategoryFilter,
    pub sort_column: SortColumn,
    pub sort_descending: bool,
}

impl App {
    pub fn new(config: AppConfig) -> App {
        let parser = HtmlParser::new(config.clone());
        App {
            state: AppState::CookieInput,
            cookies: String::new(),
            discounts: Vec::new(),
            category_list: CategoryList::new(),
            discount_list: DiscountList::new(),
            selected_discount_details: None,
            selected_discount_code: None,
            cli: None,
            config,
            parser,
            clipboard: arboard::Clipboard::new().ok(),
            error_message: None,
            search_state: SearchState::None,
            category_filter: CategoryFilter::All,
            sort_column: SortColumn::Name,
            sort_descending: false,
        }
    }

    pub fn sort_discounts(&mut self) {
        let col = self.sort_column;
        let desc = self.sort_descending;
        
        self.discounts.sort_by(|a, b| {
            let cmp = match col {
                SortColumn::Category => a.0.cmp(&b.0),
                SortColumn::Name => a.1.name.cmp(&b.1.name),
                SortColumn::Deal => {
                    let val_a = match &a.2 { Some(DealType::Percentage(p)) => *p, _ => -1 };
                    let val_b = match &b.2 { Some(DealType::Percentage(p)) => *p, _ => -1 };
                    val_a.cmp(&val_b)
                }
                SortColumn::StartDate => a.1.start_date.cmp(&b.1.start_date),
                SortColumn::EndDate => a.1.end_date.cmp(&b.1.end_date),
            };
            
            if desc { cmp.reverse() } else { cmp }
        });
    }

    pub fn sync_filtered_discounts(&mut self) {
        let indices = self.get_current_discount_indices();
        self.discount_list.update_indices(indices);
    }

    fn get_current_discount_indices(&self) -> Vec<usize> {
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
        info!("Fetching initial data with provided cookies");
        let cli = BeniCli::new(self.config.clone(), self.cookies.clone());
        match cli.fetch_discounts().await {
            Ok(response) => {
                info!("Successfully fetched {} categories", response.function_data.result.discount_structure.categories.len());
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
                
                self.discounts = all_discounts;
                self.category_list.set_items(categories);
                self.cli = Some(cli);
                self.state = AppState::CategoryList;
                Ok(())
            }
            Err(e) => {
                error!("App failed to fetch initial discounts: {}", e);
                self.error_message = Some(format!("Error fetching discounts: {}", e));
                Err(e)
            }
        }
    }

    pub async fn fetch_details(&mut self) -> Result<(), Box<dyn Error>> {
        let index = match self.discount_list.state.selected() {
            Some(i) => i,
            None => return Ok(()),
        };

        if index >= self.discount_list.filtered_indices.len() {
            error!("Selected index {} out of bounds for filtered discounts (len: {})", index, self.discount_list.filtered_indices.len());
            return Ok(());
        }

        let actual_idx = self.discount_list.filtered_indices[index];
        let discount_id = self.discounts[actual_idx].1.id;
        
        let cli = match &self.cli {
            Some(c) => c,
            None => {
                error!("BeniCli client not initialized when fetching details");
                return Ok(());
            }
        };

        info!("Fetching details for discount ID: {}", discount_id);
        let details = match cli.fetch_discount_item(discount_id).await {
            Ok(d) => d,
            Err(e) => {
                error!("App failed to fetch discount item {}: {}", discount_id, e);
                self.error_message = Some(format!("Error fetching details: {}", e));
                return Err(e);
            }
        };

        let html = &details.function_data.result.description_long_html;
        self.selected_discount_code = None;
        
        if self.parser.has_discount_code(html) {
            info!("Discount item {} has a code, fetching it", discount_id);
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