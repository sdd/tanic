use num_format::SystemLocale;
use ratatui::prelude::*;
use treemap::{MapItem, Mappable, Rect as TreeMapRect, TreemapLayout};

use crate::component::Component;

pub(crate) struct TreeMapLayout<T: Component> {
    children: Vec<(T, usize)>,
}

impl<T: Component> TreeMapLayout<T> {
    pub(crate) fn new(children: Vec<(T, usize)>) -> Self {
        Self { children }
    }
}

impl<T: Component> Component for &TreeMapLayout<T> {
    fn render(&self, area: Rect, buf: &mut Buffer, locale: &SystemLocale) {
        let layout = TreemapLayout::new();
        let bounds = TreeMapRect::from_points(
            area.x as f64,
            area.y as f64,
            area.width as f64,
            area.height as f64,
        );

        let mut regions: Vec<Box<dyn Mappable>> = self
            .children
            .iter()
            .map(|&(_, size)| {
                let res: Box<dyn Mappable> = Box::new(MapItem::with_size(size.max(1) as f64));
                res
            })
            .collect::<Vec<_>>();

        layout.layout_items(&mut regions, bounds);

        for ((child, _), region) in self.children.iter().zip(regions.iter()) {
            let region_bounds = region.bounds();

            let rect = Rect {
                x: region_bounds.x as u16,
                y: region_bounds.y as u16,
                width: region_bounds.w as u16,
                height: region_bounds.h as u16,
            };

            child.render(rect, buf, locale);
        }
    }
}
