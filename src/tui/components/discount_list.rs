use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};
use crate::beni_cli::DealType;
use crate::benifex::discount_response::Discount;
use crate::tui::app::SearchState;

pub struct DiscountListWidget<'a> {
    pub app_discounts: &'a [(String, Discount, Option<DealType>)],
    pub discount_indices: &'a [usize],
    pub is_all_discounts: bool,
    pub search_state: &'a SearchState,
}

impl<'a> StatefulWidget for DiscountListWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<ListItem> = self.discount_indices
            .iter()
            .map(|&idx| {
                let (cat, d, deal) = &self.app_discounts[idx];
                let mut spans = vec![
                    Span::styled(format!("[{}] ", cat), Style::default().fg(Color::LightBlue)),
                    Span::styled(d.name.clone(), Style::default().fg(Color::Yellow)),
                ];

                if let Some(deal_type) = deal {
                    let deal_text = match deal_type {
                        DealType::Percentage(p) => format!(" - Deal: {}%", p),
                        DealType::Trial(t) => format!(" - Deal: {} Trial", t),
                    };
                    spans.push(Span::styled(
                        deal_text,
                        Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
                    ));
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let mut list_title = if self.is_all_discounts {
            " All Discounts ".to_string()
        } else {
            " Category Discounts ".to_string()
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

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(list_title)
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        StatefulWidget::render(list, area, buf, state);
    }
}