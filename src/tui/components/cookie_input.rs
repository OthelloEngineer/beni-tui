use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

pub struct CookieInputWidget<'a> {
    pub cookies: &'a str,
}

impl<'a> Widget for CookieInputWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        let input = Paragraph::new(self.cookies)
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Paste Cookies and press Enter ")
                    .border_style(Style::default().fg(Color::LightBlue)),
            );
        Widget::render(input, chunks[0], buf);

        let guide_text = vec![
            Line::from(vec![Span::styled("--- Quick Guide to getting Cookies ---", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))]),
            Line::from(vec![]),
            Line::from(vec![Span::raw("1. Log in to your account at benify.com in your browser.")]),
            Line::from(vec![Span::raw("2. Open Developer Tools (Press "), Span::styled("F12", Style::default().fg(Color::Cyan)), Span::raw(" or "), Span::styled("Ctrl+Shift+I", Style::default().fg(Color::Cyan)), Span::raw(").")]),
            Line::from(vec![Span::raw("3. Go to the "), Span::styled("Network", Style::default().fg(Color::Cyan)), Span::raw(" tab.")]),
            Line::from(vec![Span::raw("4. Refresh the page or click any link.")]),
            Line::from(vec![Span::raw("5. Click on any request (e.g., 'fetchStructure') and find "), Span::styled("Cookie", Style::default().fg(Color::Cyan)), Span::raw(" in the "), Span::styled("Request Headers", Style::default().fg(Color::Cyan)), Span::raw(".")]),
            Line::from(vec![Span::raw("6. Copy the value, come back here, and paste it.")]),
            Line::from(vec![]),
            Line::from(vec![Span::styled("Pro-tip:", Style::default().fg(Color::LightGreen)), Span::raw(" You can also run "), Span::styled("copy(document.cookie)", Style::default().fg(Color::Yellow)), Span::raw(" in the browser Console.")]),
            Line::from(vec![]),
            Line::from(vec![Span::styled("Press Esc to Quit", Style::default().fg(Color::DarkGray))]),
        ];

        let guide = Paragraph::new(guide_text)
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: true });
        Widget::render(guide, chunks[1], buf);
    }
}