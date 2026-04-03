use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::tui::app::{App, AppState, SearchState};
use crate::tui::components::{
    category_list::CategoryListWidget, cookie_input::CookieInputWidget,
    discount_details::DiscountDetailsWidget, discount_list::DiscountListWidget,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_title(f, chunks[0]);

    match app.state {
        AppState::CookieInput => render_cookie_input(f, app, chunks[1]),
        AppState::CategoryList => render_category_list(f, app, chunks[1]),
        AppState::DiscountList | AppState::DiscountDetails => render_main_view(f, app, chunks[1]),
    }

    render_footer(f, app, chunks[2]);
}

fn render_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(" Benify Discount Browser ")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::LightBlue)));
    f.render_widget(title, area);
}

fn render_cookie_input(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_widget(CookieInputWidget { cookies: &app.cookies }, area);
}

fn render_category_list(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        CategoryListWidget { categories: &app.category_list.items },
        area,
        &mut app.category_list.state,
    );
}

fn render_main_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_details = app.state == AppState::DiscountDetails;
    let has_details = app.selected_discount_details.is_some();

    let display_chunks = if is_details && has_details {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(area)
    };

    f.render_stateful_widget(
        DiscountListWidget {
            app_discounts: &app.discounts,
            discount_indices: &app.discount_list.filtered_indices,
            category_filter: &app.category_filter,
            search_state: &app.search_state,
        },
        display_chunks[0],
        &mut app.discount_list.state,
    );

    if is_details {
        if let Some(details) = &app.selected_discount_details {
            let selected_idx_in_filtered = app.discount_list.state.selected().unwrap_or(0);
            let actual_discount_idx = app.discount_list.filtered_indices.get(selected_idx_in_filtered).copied().unwrap_or(0);
            let category_name = &app.discounts[actual_discount_idx].0;
            
            f.render_widget(
                DiscountDetailsWidget {
                    details,
                    category_name,
                    discount_code: app.selected_discount_code.as_ref(),
                    parser: &app.parser,
                },
                display_chunks[1],
            );
        }
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_text = if let Some(err) = &app.error_message {
        format!("ERROR: {}", err)
    } else {
        match app.state {
            AppState::CookieInput => "Esc: Quit | Enter: Fetch ".to_string(),
            AppState::CategoryList => "q/Esc: Quit | Enter: Browse Category | c: Change Cookies".to_string(),
            AppState::DiscountList => {
                if let SearchState::Typing(_) = app.search_state {
                    "Esc/Enter: Exit Search | Type to search".to_string()
                } else {
                    "q/Esc/Backspace: Back to Categories | Enter: Details | s: Search | n: Sort Name | g: Group | p: Sort % | d: Sort Date".to_string()
                }
            }
            AppState::DiscountDetails => {
                let mut base = "Esc/Backspace/q: Back".to_string();
                if let Some(details) = &app.selected_discount_details {
                    let html = &details.function_data.result.description_long_html;
                    if let Some(_url) = app.parser.extract_link(html) {
                        if app.parser.has_discount_code(html) {
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
    f.render_widget(footer, area);
}