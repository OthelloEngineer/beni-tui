use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use crate::benifex::discount_view::DiscountView;
use crate::beni_cli::HtmlParser;

pub struct DiscountDetailsWidget<'a> {
    pub details: &'a DiscountView,
    pub category_name: &'a str,
    pub discount_code: Option<&'a String>,
    pub parser: &'a HtmlParser,
}

impl<'a> Widget for DiscountDetailsWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut text = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::LightBlue)),
                Span::styled(&self.details.function_data.result.name, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Category: ", Style::default().fg(Color::LightBlue)),
                Span::styled(self.category_name, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled("Highlight: ", Style::default().fg(Color::LightBlue)),
                Span::styled(&self.details.function_data.result.description_highlight, Style::default().fg(Color::LightGreen)),
            ]),
            Line::from(vec![]),
            Line::from(vec![Span::styled("Description:", Style::default().fg(Color::LightBlue))]),
            Line::from(vec![Span::styled(&self.details.function_data.result.description, Style::default().fg(Color::Yellow))]),
        ];

        let html = &self.details.function_data.result.description_long_html;
        let paragraphs = self.parser.extract_paragraphs(html);

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

        if let Some(code) = self.discount_code {
            text.push(Line::from(vec![]));
            text.push(Line::from(vec![
                Span::styled("Discount Code: ", Style::default().fg(Color::LightBlue)),
                Span::styled(code.clone(), Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
            ]));
        }

        if let Some(_url) = self.parser.extract_link(html) {
            text.push(Line::from(vec![]));

            let action_text = if self.parser.has_discount_code(html) {
                "[ Press Enter or 'o' to copy code to clipboard & open website ]".to_string()
            } else {
                "[ Press Enter or 'o' to open deal website in browser ]".to_string()
            };

            text.push(Line::from(vec![Span::styled(action_text, Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD))]));
        }

        let details_para = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Discount Details ")
                    .border_style(Style::default().fg(Color::LightBlue)),
            )
            .wrap(Wrap { trim: true });
            
        Widget::render(details_para, area, buf);
    }
}