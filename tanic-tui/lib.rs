use crossterm::event::{Event, EventStream};
use ratatui::Frame;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender as MpscSender;
use tokio::sync::watch::Receiver as WatchReceiver;
use tokio_stream::{wrappers::WatchStream, StreamExt};

use crate::ui_components::app_container::AppContainer;
use tanic_core::{Result, TanicError};
use tanic_svc::state::TanicUiState;
use tanic_svc::{TanicAction, TanicAppState};

mod component;
mod ui_components;

pub struct TanicTui {
    action_tx: MpscSender<TanicAction>,
}

impl TanicTui {
    pub fn new(action_tx: MpscSender<TanicAction>) -> Self {
        Self { action_tx }
    }

    pub async fn event_loop(
        self,
        state_rx: WatchReceiver<()>,
        state: Arc<RwLock<TanicAppState>>,
    ) -> Result<()> {
        let mut terminal = ratatui::init();
        let mut term_event_stream = EventStream::new();
        let mut state_stream = WatchStream::new(state_rx);

        let Some(_) = state_stream.next().await else {
            return Ok(());
        };

        let ui = AppContainer::new(state.clone());

        loop {
            {
                let state = state.read().unwrap();
                if matches!(state.ui, TanicUiState::Exiting) {
                    break;
                }

                terminal.draw(|frame| self.draw(frame, &ui))?;
            };

            tokio::select! {
                // Catch and handle crossterm events
                maybe_event = term_event_stream.next() => match maybe_event {
                    Some(Ok(Event::Key(key)))  => {
                        if let Some(action) = ui.handle_key_event(key) {
                            self.action_tx.send(action)
                                .map_err(|err| TanicError::UnexpectedError(
                                      err.to_string()
                                ))?;
                        }
                    },
                    None => break,
                    _ => (),
                },

                // Handle state updates
                _ = state_stream.next() => {}
            }
        }

        ratatui::restore();
        Ok(())
    }

    fn draw(&self, frame: &mut Frame, ui: &AppContainer) {
        frame.render_widget(ui, frame.area());
    }
}
