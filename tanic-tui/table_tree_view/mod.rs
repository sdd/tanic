use crate::ui_state::{NamespaceTreeViewState, TableTreeViewState};
use ratatui::prelude::Stylize;
use ratatui::prelude::*;
use ratatui::widgets::canvas::{Canvas, Rectangle};
use ratatui::widgets::Block;
use treemap::{MapItem, Mappable, Rect as TreeMapRect, TreemapLayout};

use tui_logger::{LevelFilter, TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};

// find more at https://www.nerdfonts.com/cheat-sheet
const NERD_FONT_ICON_TABLE_FOLDER: &str = "\u{f12e4}"; // 󱋤
const NERD_FONT_ICON_TABLE: &str = "\u{ebb7}"; // 

pub(crate) struct TableTreeviewState {
    items: Vec<TableTreeviewItem>,
    selected: Option<usize>,
}

pub(crate) struct TableTreeviewItem {
    name: String,
    size: usize,
}

pub(crate) fn render_table_treeview(view_state: &TableTreeViewState, area: Rect, buf: &mut Buffer) {
    let [top, bottom] = Layout::vertical([Constraint::Fill(1), Constraint::Max(6)]).areas(area);

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

    let mut layout = TreemapLayout::new();
    let bounds = TreeMapRect::from_points(
        top.x as f64,
        top.y as f64,
        top.width as f64,
        top.height as f64,
    );

    let mut items: Vec<Box<dyn Mappable>> = view_state
        .tables
        .iter()
        .map(|table| {
            let res: Box<dyn Mappable> = Box::new(MapItem::with_size(table.row_count as f64));
            res
        })
        .collect::<Vec<_>>();

    layout.layout_items(&mut items, bounds);

    let selected_idx = view_state.selected_idx;

    let canvas = Canvas::default()
        .block(Block::bordered().title(format!(" Tanic //// {} Namespace ", view_state.namespace)))
        .x_bounds([top.x as f64, (top.x + top.width) as f64])
        .y_bounds([top.y as f64, (top.y + top.height) as f64])
        .paint(|ctx| {
            for (idx, item) in items.iter().enumerate() {
                let item_bounds = item.bounds();

                let rect = Rectangle {
                    x: item_bounds.x,
                    y: item_bounds.y,
                    width: item_bounds.w,
                    height: item_bounds.h,
                    color: Color::White,
                };

                ctx.draw(&rect);

                let style = if idx == selected_idx {
                    Style::new().black().bold().on_white()
                } else {
                    Style::new().white()
                };

                let name = view_state.tables[idx].name.clone();
                let name = format!("{} {}", NERD_FONT_ICON_TABLE, name);

                let name_len = name.len();
                let text = Line::styled(name, style);

                ctx.print(
                    item_bounds.x + (item_bounds.w * 0.5) - (name_len as f64 * 0.5),
                    item_bounds.y + (item_bounds.h * 0.5),
                    text,
                );
            }
        });

    canvas.render(top, buf);
}
