use crate::ui_state::ConnectionListViewState;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};

pub(crate) fn render_view_connection_list(
    state: &ConnectionListViewState,
    area: Rect,
    buf: &mut Buffer,
) {
    let title = Line::from(" Tanic ".bold());

    let block = Block::bordered()
        .title(title.centered())
        .border_set(border::THICK);

    Paragraph::new("TODO: Connection list")
        .centered()
        .block(block)
        .render(area, buf);
}
