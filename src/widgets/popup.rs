use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{block::Position, Block, BorderType, Borders, Clear, Paragraph, Widget},
};

pub enum PopUpAlignment {
    Left,
    Right,
    Center,
    Formatted,
}
///A simple PopUp. It's not designed to handle any kind of input but instead simply show something
///Before drawing it clears the contents where it's located
pub struct PopUp {
    title: String,
    content: String,

    borders: Borders,
    border_style: BorderType,

    alignment: PopUpAlignment,

    title_color: Color,
    content_color: Color,
    border_color: Color,
}

impl PopUp {
    pub fn new<T: Into<String>>(title: T, content: T) -> Self {
        Self {
            title: title.into(),
            content: content.into(),

            borders: Borders::NONE,
            border_style: BorderType::Plain,

            alignment: PopUpAlignment::Left,

            title_color: Color::from_u32(0xffffff),
            content_color: Color::from_u32(0xffffff),
            border_color: Color::from_u32(0xffffff),
        }
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn title_mut(&mut self) -> &mut String {
        &mut self.title
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }

    pub fn set_borders(&mut self, borders: Borders) {
        self.borders |= borders;
    }

    pub fn set_border_style(&mut self, style: BorderType) {
        self.border_style = style;
    }

    pub fn set_alignment(&mut self, align: PopUpAlignment) {
        self.alignment = align;
    }

    pub fn set_title_color(&mut self, color: Color) {
        self.title_color = color
    }

    pub fn set_content_color(&mut self, color: Color) {
        self.content_color = color;
    }

    pub fn set_border_color(&mut self, color: Color) {
        self.border_color = color;
    }
}
impl Widget for PopUp {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);
        let block = Block::new()
            .borders(self.borders)
            .title_position(Position::Top)
            .border_type(self.border_style)
            .title_alignment(ratatui::layout::Alignment::Center)
            .title_style(Style::new().fg(self.title_color))
            .border_style(Style::new().fg(self.border_color));
        Paragraph::new({
            //this will suppose font is monospace
            let mut out = vec![Line::default()];
            let mut idx = 0;
            let mut x = area.x;
            for content in self.content.split(' ') {
                let len = content.len() as u16;
                x += len;
                if x > area.width {
                    out.push(Line::default());
                    x = area.x;
                    idx += 1;
                }
                out[idx].push_span(Span::from(content));
            }
            out
        })
        .block(block)
        .style(Style::new().fg(self.content_color))
        .render(area, buf);
    }
}
