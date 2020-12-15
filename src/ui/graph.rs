use crate::ui::painting::*;
use crate::ui::protocol::UIBtnMode;
use crate::ui::element::{UIElement, UIElementData};
use crate::ui::constants::*;
use crate::ui::util::{draw_centered_text};

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

    fn draw_value(&self, cr: &cairo::Context, x: f64, y: f64,
                  highlight: HLStyle, data: &dyn UIElementData,
                  value: f64, val_s: &str) {

        let (w, h) = self.size();
        let h = h - UI_ELEM_TXT_H;
        let w = w - 2.0 * UI_GRPH_BORDER;
        let h = h - 2.0 * UI_GRPH_BORDER;
        let (xo, yo) = (UI_GRPH_BORDER, UI_GRPH_BORDER);

        cr.set_line_width(1.0);
        cr.set_source_rgb(
            UI_BTN_TXT_CLR.0,
            UI_BTN_TXT_CLR.1,
            UI_BTN_TXT_CLR.2);

        let name = &data.as_graph_data().unwrap().label;
        let data = data.as_graph_data().unwrap().data.borrow();

        cr.save();
        let mut first = true;
        for p in data.iter() {
            let px = p.0 as f64;
            let py = 1.0 - (p.1 as f64);
            if first {
                cr.move_to(x + xo + px * w, y + yo + py * h);
            } else {
                cr.line_to(x + xo + px * w, y + yo + py * h);
            }
            first = false;
        }
        cr.stroke();
        cr.restore();

        cr.set_font_size(self.font_size);
        draw_centered_text(
            cr, x, y + self.size().1 - UI_ELEM_TXT_H, w, UI_ELEM_TXT_H, name);
    }

    fn draw_bg(&self, cr: &cairo::Context) {
        let (w, h) = self.size();
        let h = h - UI_ELEM_TXT_H;

        cr.set_source_rgb(
            UI_BTN_BG_CLR.0,
            UI_BTN_BG_CLR.1,
            UI_BTN_BG_CLR.2);
        cr.rectangle(0.0, 0.0, w, h + UI_ELEM_TXT_H);
        cr.fill();

        cr.set_line_width(UI_GRPH_BORDER);
        cr.set_source_rgb(
            UI_GRPH_BORDER_CLR.0,
            UI_GRPH_BORDER_CLR.1,
            UI_GRPH_BORDER_CLR.2);
        let mid_border = UI_GRPH_BORDER / 2.0;
        cr.rectangle(mid_border, mid_border, w - UI_GRPH_BORDER, h - UI_GRPH_BORDER);
        cr.stroke();
    }
}
