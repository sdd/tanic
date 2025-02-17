use crate::component::Component;
use crate::ui_components::{
    namespace_list_view::NamespaceListView, splash_screen::SplashScreen,
    table_list_view::TableListView,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Color, Style, Widget};
use ratatui::widgets::Block;
use std::sync::{Arc, RwLock, TryLockError};
use tanic_svc::state::TanicUiState;
use tanic_svc::{TanicAction, TanicAppState};
use tui_logger::{LevelFilter, TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};

pub(crate) struct AppContainer {
    state: Arc<RwLock<TanicAppState>>,
    namespace_list_view: NamespaceListView,
    table_list_view: TableListView,
    splash_screen: SplashScreen,
}

impl AppContainer {
    pub(crate) fn new(state: Arc<RwLock<TanicAppState>>) -> Self {
        Self {
            state: state.clone(),

            namespace_list_view: NamespaceListView::new(state.clone()),
            table_list_view: TableListView::new(state.clone()),
            splash_screen: SplashScreen::new(state.clone()),
        }
    }

    pub(crate) fn handle_key_event(&self, key_event: KeyEvent) -> Option<TanicAction> {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('q'),
                ..
            } => {
                // User pressed Q. Dispatch an exit action
                Some(TanicAction::Exit)
            }

            key_event => {
                tracing::debug!("key_event self.state.read");
                let state = match self.state.read() {
                    Ok(state) => state,
                    Err(err) => {
                        tracing::error!(?err, %err, "poison ☠");
                        panic!();
                    }
                };
                tracing::debug!("key_event self.state.read done");

                match state.ui {
                    TanicUiState::ViewingNamespacesList(_) => {
                        (&self.namespace_list_view).handle_key_event(key_event)
                    }
                    TanicUiState::ViewingTablesList(_) => {
                        (&self.table_list_view).handle_key_event(key_event)
                    }
                    _ => None,
                }
            }
        }
    }
}

impl Widget for &AppContainer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [top, bottom] =
            Layout::vertical([Constraint::Fill(1), Constraint::Max(20)]).areas(area);

        let filter_state = TuiWidgetState::new()
            .set_default_display_level(LevelFilter::Info)
            .set_level_for_target("tanic_svc", LevelFilter::Debug);

        TuiLoggerWidget::default()
            .block(Block::bordered().title("Log"))
            .output_separator('|')
            .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Long))
            .output_target(false)
            .output_file(false)
            .output_line(false)
            .style(Style::default().fg(Color::White))
            .state(&filter_state)
            .render(bottom, buf);

        {
            tracing::debug!("render self.state.read");
            let state = match self.state.try_read() {
                Ok(state) => state,
                Err(TryLockError::Poisoned(err)) => {
                    tracing::error!(?err, %err, "poison ☠");
                    panic!();
                }
                Err(TryLockError::WouldBlock) => {
                    tracing::error!("WouldBlock");

                    // just skip this render if we can't get a read lock
                    return;
                }
            };

            match state.ui {
                TanicUiState::SplashScreen => self.splash_screen.render(top, buf),
                TanicUiState::ViewingNamespacesList(_) => {
                    (&self.namespace_list_view).render(top, buf)
                }
                TanicUiState::ViewingTablesList(_) => (&self.table_list_view).render(top, buf),
                TanicUiState::Exiting => {} // _ => {}
            }
        }
        tracing::debug!("render self.state.read done");
    }
}
