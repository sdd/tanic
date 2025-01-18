use crate::ui_state::ConnectionPromptViewState;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};

pub(crate) fn render_view_connection_prompt(
    state: &ConnectionPromptViewState,
    area: Rect,
    buf: &mut Buffer,
) {
    let title = Line::from(" Tanic ".bold());

    let block = Block::bordered()
        .title(title.centered())
        .border_set(border::THICK);

    Paragraph::new("Enter Iceberg catalog connection URI")
        .centered()
        .block(block)
        .render(area, buf);
}
