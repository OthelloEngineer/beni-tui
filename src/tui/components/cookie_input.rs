use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct CookieInputWidget<'a> {
    pub cookies: &'a str,
}

impl<'a> Widget for CookieInputWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let input = Paragraph::new(self.cookies)
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Paste Cookies and press Enter (Esc to Quit, Backspace to clear) ")
                    .border_style(Style::default().fg(Color::LightBlue)),
            );
        Widget::render(input, area, buf);
    }
}