use crate::component::Component;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tanic_svc::state::TableDescriptor;

const NERD_FONT_ICON_TABLE: &str = "\u{ebb7}"; // 

pub(crate) struct TableListItem<'a> {
    pub(crate) table: &'a TableDescriptor,
    pub(crate) is_selected: bool,
}

impl<'a> TableListItem<'a> {
    pub(crate) fn new(table: &'a TableDescriptor, is_selected: bool) -> Self {
        Self { table, is_selected }
    }
}

impl Component for &TableListItem<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::new().border_set(border::THICK);
        let block_inner = block.inner(area);

        if self.is_selected {
            block = block.style(Style::new().bg(Color::Cyan));
        }

        let name = self.table.name.clone();

        let row_count_str = match self.table.row_count() {
            None => "".to_string(),
            Some(1) => " (1 row)".to_string(),
            Some(n) => format!(" ({n} rows)"),
        };

        let name = format!("{} {}{}", NERD_FONT_ICON_TABLE, name, row_count_str);

        let para_rect = Rect::new(
            block_inner.x,
            block_inner.y + (block_inner.height / 2),
            block_inner.width,
            1,
        );

        let mut para = Paragraph::new(name)
            .alignment(Alignment::Center)
            .white()
            .bold();

        if self.is_selected {
            para = para.black();
        }

        block.render(area, buf);
        para.render(para_rect, buf);
    }
}
