use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;
use std::sync::{Arc, RwLock};

use crate::component::Component;
use crate::ui_components::table_list_item::TableListItem;
use crate::ui_components::treemap_layout::TreeMapLayout;
use tanic_svc::state::{
    RetrievedIcebergMetadata, TanicIcebergState, TanicUiState, ViewingTablesListState,
};
use tanic_svc::{TanicAction, TanicAppState};

pub(crate) struct TableListView {
    state: Arc<RwLock<TanicAppState>>,
}

impl TableListView {
    pub(crate) fn new(state: Arc<RwLock<TanicAppState>>) -> Self {
        Self { state }
    }
}

impl Component for &TableListView {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<TanicAction> {
        match key_event.code {
            KeyCode::Left => Some(TanicAction::FocusPrevTable),
            KeyCode::Right => Some(TanicAction::FocusNextTable),
            KeyCode::Enter => Some(TanicAction::SelectTable),
            KeyCode::Esc => Some(TanicAction::Escape),
            _ => None,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let state = self.state.read().unwrap();

        let TanicIcebergState::Connected(ref iceberg_state) = state.iceberg else {
            panic!();
        };

        let TanicUiState::ViewingTablesList(ref view_state) = state.ui else {
            panic!();
        };

        let block = Block::bordered()
            .title(format!(
                " Tanic //// {} Namespace ",
                view_state
                    .namespaces
                    .selected_idx
                    .and_then(|idx| iceberg_state.namespaces.get_index(idx))
                    .map(|(k, _)| k.to_string())
                    .unwrap_or("???".to_string())
            ))
            .border_set(border::PLAIN);
        let block_inner_area = block.inner(area);

        let items = TableListView::get_items(iceberg_state, view_state);

        let children: Vec<(&TableListItem, usize)> = items
            .iter()
            .map(|item| (item, item.table.row_count().unwrap_or(1) as usize))
            .collect::<Vec<_>>();

        let layout = TreeMapLayout::new(children);

        block.render(area, buf);
        (&layout).render(block_inner_area, buf);
    }
}

impl TableListView {
    fn get_items<'a>(
        iceberg_state: &'a RetrievedIcebergMetadata,
        view_state: &'a ViewingTablesListState,
    ) -> Vec<TableListItem<'a>> {
        let Some(ref selected_namespace) = view_state.namespaces.selected_idx else {
            return vec![];
        };

        let Some((_, namespace_desc)) = iceberg_state.namespaces.get_index(*selected_namespace)
        else {
            return vec![];
        };

        let Some(tables) = &namespace_desc.tables else {
            return vec![];
        };

        let items = tables
            .iter()
            .enumerate()
            .map(|(idx, (_, ns))| TableListItem::new(ns, Some(idx) == view_state.selected_idx))
            .collect::<Vec<_>>();

        items
    }
}
