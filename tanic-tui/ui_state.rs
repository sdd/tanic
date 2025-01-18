use crossterm::event::{KeyCode, KeyEvent};
use std::sync::{Arc, RwLock};
use tanic_core::config::ConnectionDetails;
use tanic_core::message::{NamespaceDeets, TableDeets};
use tanic_core::TanicMessage;

pub(crate) struct AddConnectionDialogState {
    name: String,
    uri: String,
}

pub(crate) enum TanicUiState {
    Initializing,
    ConnectionPrompt(ConnectionPromptViewState),
    ConnectionList(ConnectionListViewState),
    NamespaceTreeView(NamespaceTreeViewState),
    TableTreeView(TableTreeViewState),
}

pub(crate) struct ConnectionPromptViewState {
    connection_uri: String,
}

pub(crate) struct ConnectionListViewState {
    connections: Vec<ConnectionDetails>,
    add_connection_dialog_open: bool,
    add_connection_dialog_name: String,
    add_connection_dialog_uri: String,
}

pub(crate) struct NamespaceTreeViewState {
    pub(crate) namespaces: Vec<NamespaceDeets>,
    pub(crate) selected_idx: usize,
}

pub(crate) struct TableTreeViewState {
    pub(crate) namespace: String,
    pub(crate) tables: Vec<TableDeets>,
    pub(crate) selected_idx: usize,
}

impl NamespaceTreeViewState {
    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<TanicMessage> {
        match key_event.code {
            KeyCode::Char('q') => Some(TanicMessage::Exit),
            KeyCode::Left => {
                self.nav_left();
                None
            }
            KeyCode::Right => {
                self.nav_right();
                None
            }
            _ => None,
        }
    }

    fn nav_left(&mut self) {
        // TODO
    }

    fn nav_right(&mut self) {
        // TODO
    }
}
