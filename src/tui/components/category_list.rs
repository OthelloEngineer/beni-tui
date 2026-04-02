use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

pub struct CategoryListWidget<'a> {
    pub categories: &'a [String],
}

impl<'a> StatefulWidget for CategoryListWidget<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let items: Vec<ListItem> = self.categories
            .iter()
            .map(|cat| ListItem::new(Span::styled(cat.clone(), Style::default().fg(Color::Yellow))))
            .collect();
            
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Categories ")
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