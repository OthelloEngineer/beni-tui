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
        KeyCode::Char('q') | KeyCode::Esc => app.state = AppState::CookieInput, // Quit app
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
        KeyCode::Down => app.discount_list.next(),
        KeyCode::Up => app.discount_list.previous(),
        KeyCode::Enter => { let _ = app.fetch_details().await; },
        KeyCode::Char('n') => app.discounts.sort_by(|a, b| a.1.name.cmp(&b.1.name)),
        KeyCode::Char('s') => {
            let q = if let SearchState::Applied(q) = &app.search_state { q.clone() } else { String::new() };
            app.search_state = SearchState::Typing(q);
        },
        KeyCode::Char('g') => app.discounts.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.name.cmp(&b.1.name))),
        KeyCode::Char('p') => app.discounts.sort_by(|a, b| {
            let val_a = match &a.2 { Some(DealType::Percentage(p)) => *p, _ => -1 };
            let val_b = match &b.2 { Some(DealType::Percentage(p)) => *p, _ => -1 };
            val_b.cmp(&val_a).then(a.1.name.cmp(&b.1.name))
        }),
        KeyCode::Char('d') => app.discounts.sort_by(|a, b | {
            b.1.start_date.cmp(&a.1.start_date).then(a.1.name.cmp(&b.1.name))
        }),
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