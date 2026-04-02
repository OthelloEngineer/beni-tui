pub mod app;
pub mod components;
pub mod ui;

use std::error::Error;
use std::io;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use app::{App, AppState, CategoryFilter, SearchState};
use ui::ui;
use crate::beni_cli::{AppConfig, DealType};

pub async fn run_tui(config: AppConfig) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config);
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
                        let cat_name = app.categories[app.category_list_state.selected().unwrap_or(0)].clone();
                        if cat_name == "All Discounts" {
                            app.category_filter = CategoryFilter::All;
                        } else {
                            app.category_filter = CategoryFilter::Specific(cat_name);
                        }
                        
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
                AppState::DiscountList => {
                    let mut exit_typing = false;
                    let mut exit_search = false;
                    let mut update_typing = false;
                    let mut char_to_push = None;
                    let mut pop_char = false;

                    if let SearchState::Typing(_) = &app.search_state {
                        match key.code {
                            KeyCode::Esc => {
                                exit_search = true;
                                app.discount_list_state.select(Some(0));
                            }
                            KeyCode::Backspace => {
                                pop_char = true;
                                app.discount_list_state.select(Some(0));
                            }
                            KeyCode::Char(c) => {
                                char_to_push = Some(c);
                                app.discount_list_state.select(Some(0));
                            }
                            KeyCode::Enter => {
                                exit_typing = true;
                            }
                            _ => {}
                        }
                        update_typing = true;
                    }

                    if update_typing {
                        if exit_search {
                            app.search_state = SearchState::None;
                        } else if let SearchState::Typing(mut q) = app.search_state.clone() {
                            if pop_char {
                                q.pop();
                            }
                            if let Some(c) = char_to_push {
                                q.push(c);
                            }
                            if exit_typing {
                                if q.is_empty() {
                                    app.search_state = SearchState::None;
                                } else {
                                    app.search_state = SearchState::Applied(q);
                                }
                            } else {
                                app.search_state = SearchState::Typing(q);
                            }
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
                                app.state = AppState::CategoryList;
                                app.search_state = SearchState::None;
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
                            KeyCode::Char('n') => {
                                // Sort globally by name
                                app.discounts.sort_by(|a, b| a.1.name.cmp(&b.1.name));
                            }
                            KeyCode::Char('s') => {
                                // Enter search mode
                                let current_q = match &app.search_state {
                                    SearchState::Applied(q) => q.clone(),
                                    _ => String::new(),
                                };
                                app.search_state = SearchState::Typing(current_q);
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
                        }
                    }
                }
                AppState::DiscountDetails => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
                        app.state = AppState::DiscountList;
                    }
                    KeyCode::Char('o') | KeyCode::Enter => {
                        if let Some(details) = &app.selected_discount_details {
                            let html = &details.function_data.result.description_long_html;
                            
                            if let Some(url) = app.parser.extract_link(html) {
                                if let Some(code) = &app.selected_discount_code {
                                    if let Some(clipboard) = &mut app.clipboard {
                                        let _ = clipboard.set_text(code.clone());
                                    }
                                }

                                let _ = open::that_detached(url);
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}