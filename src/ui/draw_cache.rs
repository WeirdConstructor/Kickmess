use crate::ui::painting::*;
use crate::ui::constants::*;
use crate::ui::element::{UIElement, UIElementData};
use crate::ui::util::draw_left_text;

pub struct DrawCache {
//    surf:       Vec<Option<cairo::Surface>>,
    elements:   Vec<Box<dyn UIElement>>,
}

impl DrawCache {
    pub fn new() -> Self {
        Self {
            surf:       vec![],
            elements:   vec![],
        }
    }

    pub fn draw_container_label(&mut self, p: &dyn Painter, x: f64, y: f64, lbl: &str) {
        p.label(UI_CONT_FONT_SIZE, -1, UI_CONT_FONT_CLR, x, y, 100.0, UI_ELEM_TXT_H, lbl);
    }

    pub fn push_element(&mut self, el: Box<dyn UIElement>) {
        self.surf.push(None);
        self.elements.push(el);
    }

    pub fn size_of(&self, idx: usize) -> (f64, f64) {
        self.elements.get(idx).unwrap().size()
    }

    pub fn draw_data(&mut self, p: &dyn Painter, x: f64, y: f64,
                     idx: usize, highlight: HLStyle,
                     data: &dyn UIElementData, value: f64, val_s: &str) {
        self.elements.get(idx)
            .unwrap()
            .draw_value(p, x, y, highlight, data, value, val_s);
    }

    pub fn define_active_zones(&self, x: f64, y: f64, elem_data: &dyn UIElementData,
                               idx: usize, f: &mut dyn FnMut(ActiveZone)) {

        self.elements.get(idx).unwrap().define_active_zones(x, y, elem_data, f);
    }

    pub fn draw_bg(&mut self, p: &dyn Painter, x: f64, y: f64, idx: usize) {
        let element = self.elements.get(idx).unwrap();
        element.draw_bg(p, x, y);
    }
}

