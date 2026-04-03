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
use app::{App, AppState, CategoryFilter, SearchState, SortColumn};
use ui::ui;
use crate::beni_cli::AppConfig;

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
                AppState::CookieInput => handle_cookie_input(app, key.code).await?,
                AppState::CategoryList => handle_category_list(app, key.code),
                AppState::DiscountList => handle_discount_list(app, key.code).await?,
                AppState::DiscountDetails => handle_discount_details(app, key.code),
            }
        }
    }
}

async fn handle_cookie_input(app: &mut App, code: KeyCode) -> io::Result<()> {
    match code {
        KeyCode::Enter if !app.cookies.is_empty() => {
            let _ = app.fetch_data().await;
        }
        KeyCode::Char(c) => app.cookies.push(c),
        KeyCode::Backspace => { app.cookies.pop(); },
        KeyCode::Esc => return Err(io::Error::new(io::ErrorKind::Other, "User quit")),
        _ => {}
    }
    Ok(())
}

fn handle_category_list(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.state = AppState::CookieInput,
        KeyCode::Down => app.category_list.next(),
        KeyCode::Up => app.category_list.previous(),
        KeyCode::Enter => {
            let cat_name = app.category_list.selected_name().unwrap_or_else(|| "All Discounts".to_string());
            app.category_filter = if cat_name == "All Discounts" {
                CategoryFilter::All
            } else {
                CategoryFilter::Specific(cat_name)
            };
            app.sync_filtered_discounts();
            app.state = AppState::DiscountList;
        }
        KeyCode::Char('c') => app.state = AppState::CookieInput,
        _ => {}
    }
}

async fn handle_discount_list(app: &mut App, code: KeyCode) -> io::Result<()> {
    if let SearchState::Typing(mut q) = app.search_state.clone() {
        match code {
            KeyCode::Esc => app.search_state = SearchState::None,
            KeyCode::Backspace => { q.pop(); app.search_state = SearchState::Typing(q); },
            KeyCode::Char(c) => { q.push(c); app.search_state = SearchState::Typing(q); },
            KeyCode::Enter => app.search_state = if q.is_empty() { SearchState::None } else { SearchState::Applied(q) },
            _ => {}
        }
        app.sync_filtered_discounts();
        return Ok(());
    }

    match code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => {
            app.state = AppState::CategoryList;
            app.search_state = SearchState::None;
        }
        KeyCode::Down | KeyCode::Char('j') => app.discount_list.next(),
        KeyCode::Up | KeyCode::Char('k') => app.discount_list.previous(),
        KeyCode::Enter => { let _ = app.fetch_details().await; },
        KeyCode::Char('n') => {
            app.sort_column = SortColumn::Name;
            app.sort_discounts();
        }
        KeyCode::Char('s') => {
            let q = if let SearchState::Applied(q) = &app.search_state { q.clone() } else { String::new() };
            app.search_state = SearchState::Typing(q);
        },
        KeyCode::Char('g') => {
            app.sort_column = SortColumn::Category;
            app.sort_discounts();
        }
        KeyCode::Char('p') => {
            app.sort_column = SortColumn::Deal;
            app.sort_descending = !app.sort_descending;
            app.sort_discounts();
        }
        KeyCode::Char('h') => {
            app.sort_column = app.sort_column.previous();
            app.sort_discounts();
        }
        KeyCode::Char('l') => {
            app.sort_column = app.sort_column.next();
            app.sort_discounts();
        }
        KeyCode::Char(' ') => {
            app.sort_descending = !app.sort_descending;
            app.sort_discounts();
        }
        _ => {}
    }
    app.sync_filtered_discounts();
    Ok(())
}

fn handle_discount_details(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Backspace => app.state = AppState::DiscountList,
        KeyCode::Char('o') | KeyCode::Enter => {
            if let Some(details) = &app.selected_discount_details {
                if let Some(url) = app.parser.extract_link(&details.function_data.result.description_long_html) {
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
    }
}