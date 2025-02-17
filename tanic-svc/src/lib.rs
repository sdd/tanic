use std::sync::Arc;
use std::sync::RwLock;
use tanic_core::TanicConfig;
use tanic_core::{Result, TanicError};
use tokio::sync::mpsc::{UnboundedReceiver as MpscReceiver, UnboundedSender as MpscSender};
use tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender};

pub mod iceberg_context;
pub mod state;

use crate::state::TanicIcebergState;
pub use state::{TanicAction, TanicAppState};

pub struct AppStateManager {
    action_rx: MpscReceiver<TanicAction>,

    #[allow(unused)]
    action_tx: MpscSender<TanicAction>,
    state_tx: WatchSender<()>,

    state: Arc<RwLock<TanicAppState>>,
}

impl AppStateManager {
    pub fn new(_config: TanicConfig) -> (Self, MpscSender<TanicAction>, WatchReceiver<()>) {
        let state = Arc::new(RwLock::new(TanicAppState::default()));

        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
        let (state_tx, state_rx) = tokio::sync::watch::channel(());

        (
            Self {
                action_rx,
                action_tx: action_tx.clone(),
                state_tx,
                state,
            },
            action_tx,
            state_rx,
        )
    }

    pub fn get_state(&self) -> Arc<RwLock<TanicAppState>> {
        self.state.clone()
    }

    pub async fn event_loop(self) -> Result<()> {
        let Self {
            state,
            state_tx,
            mut action_rx,
            ..
        } = self;

        while !matches!(state.read().unwrap().iceberg, TanicIcebergState::Exiting) {
            tracing::debug!("await action_rx.recv()");
            let Some(action) = action_rx.recv().await else {
                break;
            };
            tracing::debug!("await action_rx.recv() complete");
            tracing::info!(?action, "AppState received an action");

            {
                tracing::debug!("state.write()");
                let mut mut_state = state.write().unwrap();
                *mut_state = mut_state.clone().update(action);
            }
            tracing::debug!("state.write() done");

            state_tx
                .send(())
                .map_err(|err| TanicError::UnexpectedError(err.to_string()))?;
        }

        Ok(())
    }
}
