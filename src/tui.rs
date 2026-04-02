use std::error::Error;
use std::io;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use regex::Regex;
use std::collections::HashSet;
use crate::beni_cli::{BeniCli, Config};
use crate::benifex::discount_response::Discount;
use crate::benifex::discount_parser::{DiscountParser, DealType};

#[derive(PartialEq)]
enum AppState {
    CookieInput,
    CategoryList,
    DiscountList,
    DiscountDetails,
}

pub struct App {
    state: AppState,
    cookies: String,
    categories: Vec<String>,
    discounts: Vec<(String, Discount, Option<DealType>)>, // (Category Name, Discount, DealType)
    category_list_state: ListState,
    discount_list_state: ListState,
    selected_discount_details: Option<crate::benifex::discount_view::DiscountView>,
    selected_discount_code: Option<String>,
    cli: Option<BeniCli>,
    error_message: Option<String>,
}

impl App {
    pub fn new() -> App {
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
            error_message: None,
        }
    }

    fn get_current_discounts(&self) -> Vec<&(String, Discount, Option<DealType>)> {
        if self.categories.is_empty() {
            return Vec::new();
        }
        let active_cat_name = &self.categories[self.category_list_state.selected().unwrap_or(0)];
        
        if active_cat_name == "All Discounts" {
            let mut seen = HashSet::new();
            let mut result = Vec::new();
            for d in &self.discounts {
                if seen.insert(d.1.id) {
                    result.push(d);
                }
            }
            result
        } else {
            self.discounts.iter().filter(|(c, _, _)| c == active_cat_name).collect()
        }
    }

    async fn fetch_data(&mut self) -> Result<(), Box<dyn Error>> {
        let config = Config::new(
            "https://www.benify.com".to_string(),
            "/fps/user/discount/fetchStructure".to_string(),
            "/fps/user/discount/fetch?discountId=".to_string(),
            "/fps/user/discountCode/view?discountId=".to_string(),
        );
        let cli = BeniCli::new(config, self.cookies.clone());
        let parser = DiscountParser::new(Regex::new(r"(\d+\s*(?:måneder|dage))\s*gratis").unwrap());
        match cli.fetch_discounts().await {
            Ok(response) => {
                let mut all_discounts = Vec::new();
                let mut categories = Vec::new();
                categories.push("All Discounts".to_string());
                
                for category in response.function_data.result.discount_structure.categories {
                    categories.push(category.name.clone());
                    for discount in category.discounts {
                        let deal_type = parser.parse_discount_from_highlight(&discount.description_highlight);
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

    async fn fetch_details(&mut self) -> Result<(), Box<dyn Error>> {
        let current_discounts = self.get_current_discounts();
        if let Some(index) = self.discount_list_state.selected() {
            if index < current_discounts.len() {
                let discount_id = current_discounts[index].1.id;
                if let Some(cli) = &self.cli {
                    match cli.fetch_discount_item(discount_id).await {
                        Ok(details) => {
                            let html = &details.function_data.result.description_long_html;
                            let re_code = Regex::new(r#"data-unique-discount-code-url="([^"]+)""#).unwrap();
                            self.selected_discount_code = None;
                            
                            if re_code.is_match(html) {
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

pub async fn run_tui() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend<Error = io::Error>>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.state {
                AppState::CookieInput => match key.code {
                    KeyCode::Enter => {
                        if !app.cookies.is_empty() {
                           if let Err(e) = app.fetch_data().await {
                               app.error_message = Some(format!("Failed to fetch: {}", e));
                           }
                        }
                    }
                    KeyCode::Char(c) => {
                        app.cookies.push(c);
                    }
                    KeyCode::Backspace => {
                        app.cookies.pop();
                    }
                    KeyCode::Esc => return Ok(()),
                    _ => {}
                },
                AppState::CategoryList => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Down => {
                        let i = match app.category_list_state.selected() {
                            Some(i) => {
                                if i >= app.categories.len().saturating_sub(1) {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.category_list_state.select(Some(i));
                    }
                    KeyCode::Up => {
                        let i = match app.category_list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.categories.len().saturating_sub(1)
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.category_list_state.select(Some(i));
                    }
                    KeyCode::Enter => {
                        app.state = AppState::DiscountList;
                        let len = app.get_current_discounts().len();
                        if len > 0 {
                            app.discount_list_state.select(Some(0));
                        } else {
                            app.discount_list_state.select(None);
                        }
                    }
                    KeyCode::Char('c') => {
                        app.state = AppState::CookieInput;
                    }
                    _ => {}
                },
                AppState::DiscountList => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
                        app.state = AppState::CategoryList;
                    }
                    KeyCode::Down => {
                        let current_len = app.get_current_discounts().len();
                        if current_len > 0 {
                            let i = match app.discount_list_state.selected() {
                                Some(i) => {
                                    if i >= current_len - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            app.discount_list_state.select(Some(i));
                        }
                    }
                    KeyCode::Up => {
                        let current_len = app.get_current_discounts().len();
                        if current_len > 0 {
                            let i = match app.discount_list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        current_len - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            app.discount_list_state.select(Some(i));
                        }
                    }
                    KeyCode::Enter => {
                        if let Err(e) = app.fetch_details().await {
                            app.error_message = Some(format!("Failed to fetch details: {}", e));
                        }
                    }
                    KeyCode::Char('s') => {
                        // Sort globally
                        app.discounts.sort_by(|a, b| a.1.name.cmp(&b.1.name));
                    }
                    KeyCode::Char('g') => {
                        // Sort globally by category
                        app.discounts.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.name.cmp(&b.1.name)));
                    }
                    KeyCode::Char('p') => {
                        // Sort globally by percentage
                        app.discounts.sort_by(|a, b| {
                            let val_a = match &a.2 {
                                Some(DealType::Percentage(p)) => *p,
                                _ => -1,
                            };
                            let val_b = match &b.2 {
                                Some(DealType::Percentage(p)) => *p,
                                _ => -1,
                            };
                            val_b.cmp(&val_a).then(a.1.name.cmp(&b.1.name))
                        });
                    }
                    _ => {}
                },
                AppState::DiscountDetails => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
                        app.state = AppState::DiscountList;
                    }
                    KeyCode::Char('o') | KeyCode::Enter => {
                        if let Some(details) = &app.selected_discount_details {
                            let html = &details.function_data.result.description_long_html;
                            let re_link = Regex::new(r#"data-external-link-reference="([^"]+)""#).unwrap();

                            if let Some(caps) = re_link.captures(html) {
                                let mut url = caps.get(1).unwrap().as_str().to_string();
                                url = url.replace("&amp;", "&");
                                
                                if let Some(code) = &app.selected_discount_code {
                                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                        let _ = clipboard.set_text(code.clone());
                                    }
                                }

                                let _ = open::that(url);
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    let title = Paragraph::new(" Benify Discount Browser ")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::LightBlue)));
    f.render_widget(title, chunks[0]);

    match app.state {
        AppState::CookieInput => {
            let input = Paragraph::new(app.cookies.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Paste Cookies and press Enter (Esc to Quit, Backspace to clear) ")
                    .border_style(Style::default().fg(Color::LightBlue)));
            f.render_widget(input, chunks[1]);
        }
        AppState::CategoryList => {
            let items: Vec<ListItem> = app.categories
                .iter()
                .map(|cat| {
                    ListItem::new(Span::styled(cat.clone(), Style::default().fg(Color::Yellow)))
                })
                .collect();
            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Categories ")
                    .border_style(Style::default().fg(Color::LightBlue)))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::LightYellow).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[1], &mut app.category_list_state);
        }
        AppState::DiscountList | AppState::DiscountDetails => {
            let is_details = matches!(app.state, AppState::DiscountDetails);
            let display_chunks = if is_details && app.selected_discount_details.is_some() {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .split(chunks[1])
            } else {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(100)])
                    .split(chunks[1])
            };

            let current_discounts = app.get_current_discounts();
            let items: Vec<ListItem> = current_discounts
                .iter()
                .map(|(cat, d, deal)| {
                    let mut spans = vec![
                        Span::styled(format!("[{}] ", cat), Style::default().fg(Color::LightBlue)),
                        Span::styled(d.name.clone(), Style::default().fg(Color::Yellow)),
                    ];

                    if let Some(deal_type) = deal {
                        let deal_text = match deal_type {
                            DealType::Percentage(p) => format!(" - Deal: {}%", p),
                            DealType::Trial(t) => format!(" - Deal: {} Trial", t),
                        };
                        spans.push(Span::styled(deal_text, Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)));
                    }

                    ListItem::new(Line::from(spans))
                })
                .collect();
            
            let list_title = if app.category_list_state.selected().unwrap_or(0) == 0 {
                " All Discounts "
            } else {
                " Category Discounts "
            };

            let list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(list_title)
                    .border_style(Style::default().fg(Color::LightBlue)))
                .highlight_style(Style::default().bg(Color::Blue).fg(Color::LightYellow).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, display_chunks[0], &mut app.discount_list_state);

            if is_details {
                if let Some(details) = &app.selected_discount_details {
                    let mut text = vec![
                        Line::from(vec![Span::styled("Name: ", Style::default().fg(Color::LightBlue)), Span::styled(&details.function_data.result.name, Style::default().fg(Color::Yellow))]),
                        Line::from(vec![Span::styled("Category: ", Style::default().fg(Color::LightBlue)), Span::styled(&app.discounts[app.discount_list_state.selected().unwrap_or(0)].0, Style::default().fg(Color::Yellow))]),
                        Line::from(vec![]),
                        Line::from(vec![Span::styled("Highlight: ", Style::default().fg(Color::LightBlue)), Span::styled(&details.function_data.result.description_highlight, Style::default().fg(Color::LightGreen))]),
                        Line::from(vec![]),
                        Line::from(vec![Span::styled("Description:", Style::default().fg(Color::LightBlue))]),
                        Line::from(vec![Span::styled(&details.function_data.result.description, Style::default().fg(Color::Yellow))]),
                    ];

                    let html = &details.function_data.result.description_long_html;
                    let re_link = Regex::new(r#"data-external-link-reference="([^"]+)""#).unwrap();
                    let re_code = Regex::new(r#"data-unique-discount-code-url="([^"]+)""#).unwrap();
                    let re_paragraph = Regex::new(r#"<p class="paragraph-node">([\s\S]*?)</p>"#).unwrap();
                    let re_html_tags = Regex::new(r"<[^>]*>").unwrap();

                    let mut paragraphs = Vec::new();
                    for cap in re_paragraph.captures_iter(html) {
                        let p_html = cap.get(1).unwrap().as_str();
                        // Replace <br> and <br/> with newline before stripping other tags
                        let p_br = p_html.replace("<br>", "\n").replace("<br/>", "\n");
                        let p_text = re_html_tags.replace_all(&p_br, "").trim().to_string();
                        if !p_text.is_empty() {
                            paragraphs.push(p_text);
                        }
                    }

                    if !paragraphs.is_empty() {
                        text.push(Line::from(vec![]));
                        text.push(Line::from(vec![Span::styled("More Info:", Style::default().fg(Color::LightBlue))]));
                        for p in paragraphs {
                            // Handle possible multiple lines within a single paragraph string due to <br> replacement
                            for line in p.split('\n') {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    text.push(Line::from(vec![Span::styled(trimmed.to_string(), Style::default().fg(Color::Yellow))]));
                                }
                            }
                        }
                    }

                    if let Some(code) = &app.selected_discount_code {
                        text.push(Line::from(vec![]));
                        text.push(Line::from(vec![Span::styled("Discount Code: ", Style::default().fg(Color::LightBlue)), Span::styled(code.clone(), Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD))]));
                    }

                    if re_link.is_match(html) {
                        text.push(Line::from(vec![]));
                        
                        let mut action_text = "[ Press Enter or 'o' to open deal website in browser ]".to_string();
                        if re_code.is_match(html) {
                            action_text = "[ Press Enter or 'o' to copy code to clipboard & open website ]".to_string();
                        }
                        
                        text.push(Line::from(vec![Span::styled(action_text, Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD))]));
                    }

                    let details_para = Paragraph::new(text)
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .title(" Discount Details ")
                            .border_style(Style::default().fg(Color::LightBlue)))
                        .wrap(Wrap { trim: true });
                    f.render_widget(details_para, display_chunks[1]);
                }
            }
        }
    }

    let footer_text = if let Some(err) = &app.error_message {
        format!("ERROR: {}", err)
    } else {
        match app.state {
            AppState::CookieInput => "Esc: Quit | Enter: Fetch".to_string(),
            AppState::CategoryList => "q/Esc: Quit | Enter: Browse Category | c: Change Cookies".to_string(),
            AppState::DiscountList => "q/Esc/Backspace: Back to Categories | Enter: Details | s: Name | g: Group | p: %".to_string(),
            AppState::DiscountDetails => {
                let mut base = "Esc/Backspace/q: Back".to_string();
                if let Some(details) = &app.selected_discount_details {
                    let html = &details.function_data.result.description_long_html;
                    let re_link = Regex::new(r#"data-external-link-reference="([^"]+)""#).unwrap();
                    let re_code = Regex::new(r#"data-unique-discount-code-url="([^"]+)""#).unwrap();
                    
                    if re_link.is_match(html) {
                        if re_code.is_match(html) {
                            base.push_str(" | Enter/o: Copy Code & Open");
                        } else {
                            base.push_str(" | Enter/o: Open Link");
                        }
                    }
                }
                base
            }
        }
    };
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(footer, chunks[2]);
}
