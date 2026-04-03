use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, TableState},
};
use crate::beni_cli::DealType;
use crate::benifex::discount_response::Discount;
use crate::tui::app::{CategoryFilter, SearchState, SortColumn};

pub struct DiscountListWidget<'a> {
    app_discounts: &'a [(String, Discount, Option<DealType>)],
    discount_indices: &'a [usize],
    category_filter: &'a CategoryFilter,
    search_state: &'a SearchState,
    sort_column: SortColumn,
    sort_descending: bool,
}

impl<'a> DiscountListWidget<'a> {
    pub fn new(
        app_discounts: &'a [(String, Discount, Option<DealType>)],
        discount_indices: &'a [usize],
        category_filter: &'a CategoryFilter,
        search_state: &'a SearchState,
        sort_column: SortColumn,
        sort_descending: bool,
    ) -> Self {
        Self {
            app_discounts,
            discount_indices,
            category_filter,
            search_state,
            sort_column,
            sort_descending,
        }
    }
}

impl<'a> StatefulWidget for DiscountListWidget<'a> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header_cells = [
            ("Category", SortColumn::Category),
            ("Name", SortColumn::Name),
            ("Deal", SortColumn::Deal),
            ("Start", SortColumn::StartDate),
            ("End", SortColumn::EndDate),
        ]
        .into_iter()
        .map(|(title, col)| {
            let mut text = title.to_string();
            let mut style = Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD);
            
            if self.sort_column == col {
                style = style.fg(Color::Yellow).add_modifier(Modifier::UNDERLINED);
                text.push(if self.sort_descending { '▼' } else { '▲' });
            }
            
            Cell::from(text).style(style)
        });

        let header = Row::new(header_cells)
            .style(Style::default().bg(Color::DarkGray))
            .height(1)
            .bottom_margin(1);

        let rows = self.discount_indices.iter().map(|&idx| {
            let (cat, d, deal) = &self.app_discounts[idx];
            
            let deal_text = match deal {
                Some(DealType::Percentage(p)) => format!("{}%", p),
                Some(DealType::Trial(t)) => format!("{} Trial", t),
                None => "N/A".to_string(),
            };

            let cells = vec![
                Cell::from(cat.clone()).style(Style::default().fg(Color::LightBlue)),
                Cell::from(d.name.clone()).style(Style::default().fg(Color::Yellow)),
                Cell::from(deal_text).style(Style::default().fg(Color::LightGreen)),
                Cell::from(d.start_date.clone()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(d.end_date.as_deref().unwrap_or("N/A").to_string()).style(Style::default().fg(Color::DarkGray)),
            ];
            Row::new(cells).height(1)
        });

        let mut list_title = match self.category_filter {
            CategoryFilter::All => " All Discounts ".to_string(),
            CategoryFilter::Specific(_) => " Category Discounts ".to_string(),
        };

        match self.search_state {
            SearchState::Typing(q) => {
                list_title = format!(" Search: {}█ ", q);
            }
            SearchState::Applied(q) if !q.is_empty() => {
                list_title = format!(" Search: {} ", q);
            }
            _ => {}
        }

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(15),
                Constraint::Percentage(45),
                Constraint::Percentage(15),
                Constraint::Percentage(12),
                Constraint::Percentage(12),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(Color::LightBlue)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

        StatefulWidget::render(table, area, buf, state);
    }
}