use crate::ui_state::{TanicUiState, ViewNamespaceTreeViewState};
use crossterm::event::{self, Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::stream::StreamExt;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget, Frame};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tanic_core::TanicMessage;
use tokio::sync::Mutex;

use crate::connection_list::render_view_connection_list;
use crate::connection_prompt::render_view_connection_prompt;
use crate::initializing::render_view_initializing;
use crate::namespace_tree_view::render_namespace_treeview;
use tanic_core::message::NamespaceDeets;
use tanic_core::Result;
use tokio::sync::mpsc::{Receiver, Sender};

mod connection_list;
mod connection_prompt;
mod initializing;
mod namespace_tree_view;
mod ui_state;

pub struct TanicTui {
    should_exit: AtomicBool,
    rx: Arc<Mutex<Receiver<TanicMessage>>>,
    tx: Sender<TanicMessage>,
    term_event_stream: Arc<Mutex<EventStream>>,
    state: Arc<RwLock<TanicUiState>>,
}

impl TanicTui {
    pub async fn start(rx: Receiver<TanicMessage>, tx: Sender<TanicMessage>) -> Result<()> {
        TanicTui::new(rx, tx).event_loop().await
    }

    fn new(rx: Receiver<TanicMessage>, tx: Sender<TanicMessage>) -> Self {
        Self {
            should_exit: AtomicBool::new(false),
            rx: Arc::new(Mutex::new(rx)),
            tx,
            term_event_stream: Arc::new(Mutex::new(EventStream::new())),
            state: Arc::new(RwLock::new(TanicUiState::Initializing)),
        }
    }

    async fn event_loop(&self) -> Result<()> {
        let mut terminal = ratatui::init();

        while !self.should_exit.load(Ordering::Relaxed) {
            terminal.draw(|frame| self.draw(frame))?;

            tokio::select! {
                _ = self.handle_terminal_events() => {},
                _ = self.handle_tanic_events() => {},
            }
        }

        ratatui::restore();
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    async fn handle_tanic_events(&self) -> Result<()> {
        let mut rx = self.rx.lock().await;
        let Some(message) = rx.recv().await else {
            return Ok(());
        };
        tracing::info!(?message, "tui received a tanic message");

        match message {
            TanicMessage::Exit => self.exit(),

            TanicMessage::ShowNamespaces(namespaces) => self.show_namespaces(namespaces),

            _ => {}
        }

        Ok(())
    }

    async fn handle_terminal_events(&self) -> Result<()> {
        let mut term_event_stream = self.term_event_stream.lock().await;
        let Some(Ok(message)) = term_event_stream.next().await else {
            unreachable!();
        };

        match message {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            // KeyCode::Left => self.decrement_counter(),
            // KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&self) {
        self.should_exit.store(true, Ordering::Relaxed)
    }

    fn show_namespaces(&self, namespaces: Vec<NamespaceDeets>) {
        let mut state = self.state.write().unwrap();

        *state = TanicUiState::NamespaceTreeView(ViewNamespaceTreeViewState { namespaces });
    }
}

impl Widget for &TanicTui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state.read().unwrap();

        match state.deref() {
            TanicUiState::Initializing => render_view_initializing(area, buf),
            TanicUiState::ConnectionPrompt(view_state) => {
                render_view_connection_prompt(&view_state, area, buf)
            }
            TanicUiState::ConnectionList(view_state) => {
                render_view_connection_list(&view_state, area, buf)
            }
            TanicUiState::NamespaceTreeView(view_state) => {
                render_namespace_treeview(&view_state, area, buf)
            }
        }
    }
}
