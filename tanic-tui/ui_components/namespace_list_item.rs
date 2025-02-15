use crate::component::Component;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tanic_svc::state::NamespaceDescriptor;

const NERD_FONT_ICON_TABLE_FOLDER: &str = "\u{f12e4}"; // 󱋤

pub(crate) struct NamespaceListItem<'a> {
    pub(crate) ns: &'a NamespaceDescriptor,
    pub(crate) is_selected: bool,
}

impl<'a> NamespaceListItem<'a> {
    pub(crate) fn new(ns: &'a NamespaceDescriptor, is_selected: bool) -> Self {
        Self { ns, is_selected }
    }
}

impl Component for &NamespaceListItem<'_> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::new().border_set(border::THICK);
        let block_inner = block.inner(area);

        if self.is_selected {
            block = block.style(Style::new().bg(Color::Cyan));
        }

        let name = self.ns.name.clone();
        let tables = &self.ns.tables;
        let table_count = tables.as_ref().map(|t| t.len()).unwrap_or(0);
        let plural_suffix = if table_count == 1 { "" } else { "s" };

        let row_count = self.ns.row_count.unwrap_or(0);
        let row_plural_suffix = if row_count == 1 { "" } else { "s" };

        let name = format!(
            "{} {} ({} table{}, {} row{})",
            NERD_FONT_ICON_TABLE_FOLDER,
            name,
            table_count,
            plural_suffix,
            row_count,
            row_plural_suffix
        );

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
