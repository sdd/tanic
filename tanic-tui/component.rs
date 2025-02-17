use num_format::SystemLocale;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use ratatui::prelude::*;

use tanic_svc::TanicAction;

pub trait Component {
    fn handle_key_event(&mut self, _key: KeyEvent) -> Option<TanicAction> {
        None
    }

    #[allow(dead_code)] // not using any mouse events yet
    fn handle_mouse_event(&mut self, _mouse: MouseEvent) -> Option<TanicAction> {
        None
    }

    fn render(&self, area: Rect, buf: &mut Buffer, locale: &SystemLocale);
}
