use crate::ui::painting::*;
use crate::ui::protocol::UIBtnMode;
use crate::ui::element::{UIElement, UIElementData};
use crate::ui::constants::*;

pub struct Graph {
    w:          f64,
    h:          f64,
    font_size:  f64,
}

impl Graph {
    pub fn new(w: f64, h: f64, font_size: f64) -> Self {
        Self {
            w,
            h: h + UI_ELEM_TXT_H,
            font_size,
        }
    }
}


impl UIElement for Graph {
    fn size(&self) -> (f64, f64) {
        (self.w, self.h)
    }

    fn define_active_zones(&self, _x: f64, _y: f64,
                           _elem_data: &dyn UIElementData,
                           _f: &mut dyn FnMut(ActiveZone)) {
    }

    fn draw_value(&self, p: &mut dyn Painter, x: f64, y: f64,
                  highlight: HLStyle, data: &dyn UIElementData,
                  value: f64, val_s: &str) {

        let (w, h) = self.size();
        let h = h - UI_ELEM_TXT_H;
        let w = w - 2.0 * UI_GRPH_BORDER;
        let h = h - 2.0 * UI_GRPH_BORDER;
        let (xo, yo) = (UI_GRPH_BORDER, UI_GRPH_BORDER);

        let name = &data.as_graph_data().unwrap().label;
        let data = data.as_graph_data().unwrap().data.borrow();

        p.path_stroke(
            1.0, UI_BTN_TXT_CLR,
            &mut (data.iter().map(|p: &(f64, f64)| (p.0 * w + xo + x, p.1 * h + yo + y))),
            false);
        p.label(
            self.font_size, -1, UI_BTN_TXT_CLR,
            x, y + self.size().1 - UI_ELEM_TXT_H,
            w, UI_ELEM_TXT_H, name);
    }

    fn draw_bg(&self, p: &mut dyn Painter, x: f64, y: f64) {
        let (w, h) = self.size();
        let h = h - UI_ELEM_TXT_H;

        p.rect_fill(UI_BTN_BG_CLR, x, y, w, h + UI_ELEM_TXT_H);

        let mid_border = UI_GRPH_BORDER / 2.0;
        p.rect_stroke(
            UI_GRPH_BORDER, UI_GRPH_BORDER_CLR,
            x + mid_border, y + mid_border, w - UI_GRPH_BORDER, h - UI_GRPH_BORDER);
    }
}
