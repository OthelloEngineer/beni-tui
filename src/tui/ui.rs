use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::tui::app::{App, AppState};
use crate::beni_cli::DealType;

pub fn ui(f: &mut Frame, app: &mut App) {
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
                    
                    let paragraphs = app.parser.extract_paragraphs(html);

                    if !paragraphs.is_empty() {
                        text.push(Line::from(vec![]));
                        text.push(Line::from(vec![Span::styled("More Info:", Style::default().fg(Color::LightBlue))]));
                        for p in paragraphs {
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

                    if let Some(_url) = app.parser.extract_link(html) {
                        text.push(Line::from(vec![]));
                        
                        let mut action_text = "[ Press Enter or 'o' to open deal website in browser ]".to_string();
                        if app.parser.has_discount_code(html) {
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
            AppState::DiscountList => {
                if app.search_mode {
                    "Esc/Enter: Exit Search | Type to search".to_string()
                } else {
                    "q/Esc/Backspace: Back to Categories | Enter: Details | s: Search | n: Sort Name | g: Group | p: Sort %".to_string()
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
    f.render_widget(footer, chunks[2]);
}