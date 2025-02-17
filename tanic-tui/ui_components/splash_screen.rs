use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use std::sync::{Arc, RwLock};
use tanic_svc::TanicAppState;

pub(crate) struct SplashScreen {
    _state: Arc<RwLock<TanicAppState>>,
}

impl SplashScreen {
    pub(crate) fn new(state: Arc<RwLock<TanicAppState>>) -> Self {
        Self { _state: state }
    }
}

impl Widget for &SplashScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::new().white().bold();
        let title = Line::styled(" Tanic ".to_string(), style);

        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let counter_text = Text::from(vec![Line::from(vec!["Initializing...".into()])]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
