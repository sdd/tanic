use crate::ui_state::ViewNamespaceTreeViewState;
use ratatui::prelude::*;
use ratatui::widgets::canvas::{Canvas, Rectangle};
use ratatui::widgets::Block;
use treemap::{MapItem, Mappable, Rect as TreeMapRect, TreemapLayout};

use tui_logger::{LevelFilter, TuiLoggerLevelOutput, TuiLoggerWidget, TuiWidgetState};

pub(crate) struct NamespaceTreeviewState {
    items: Vec<NamespaceTreeviewItem>,
    selected: Option<usize>,
}

pub(crate) struct NamespaceTreeviewItem {
    name: String,
    size: usize,
}

pub(crate) fn render_namespace_treeview(
    view_state: &ViewNamespaceTreeViewState,
    area: Rect,
    buf: &mut Buffer,
) {
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
        .namespaces
        .iter()
        .map(|ns| {
            let res: Box<dyn Mappable> = Box::new(MapItem::with_size(1.0));
            res
        })
        .collect::<Vec<_>>();

    layout.layout_items(&mut items, bounds);

    let canvas = Canvas::default()
        .block(Block::bordered().title(" Tanic /// Namespaces "))
        .x_bounds([top.x as f64, (top.x + top.width) as f64])
        .y_bounds([top.y as f64, (top.y + top.height) as f64])
        .paint(|ctx| {
            for (idx, item) in items.iter().enumerate() {
                let item_bounds = item.bounds();
                ctx.draw(&Rectangle {
                    x: item_bounds.x,
                    y: item_bounds.y,
                    width: item_bounds.w,
                    height: item_bounds.h,
                    color: Color::Red,
                });

                ctx.print(
                    item_bounds.x + (item_bounds.w * 0.5),
                    item_bounds.y + (item_bounds.h * 0.5),
                    (&view_state.namespaces[idx].name).to_string(),
                );
            }
        });

    canvas.render(top, buf);
}
