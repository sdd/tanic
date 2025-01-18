use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};

pub(crate) fn render_view_initializing(area: Rect, buf: &mut Buffer) {
    let title = Line::from(" Tanic ".bold());

    let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);

    let block = Block::bordered()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .border_set(border::THICK);

    let counter_text = Text::from(vec![Line::from(vec!["Initializing...".into()])]);

    Paragraph::new(counter_text)
        .centered()
        .block(block)
        .render(area, buf);
}
