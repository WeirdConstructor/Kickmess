use crate::ui::painting::*;
use crate::ui::protocol::UIBtnMode;
use crate::ui::element::{UIElement, UIElementData};
use crate::ui::constants::*;
use crate::ui::util::{draw_centered_text};

pub struct Graph {
}

impl Graph {
    pub fn new() -> Self {
        Self {
        }
    }
}


impl UIElement for Graph {
    fn size(&self) -> (f64, f64) {
        (90.0, 40.0)
    }

    fn define_active_zones(&self, x: f64, y: f64, elem_data: &dyn UIElementData, f: &mut dyn FnMut(ActiveZone)) {
//        let size     = self.size();
//        let sub_type =
//            match elem_data.as_btn_data().unwrap().mode {
//                UIBtnMode::Toggle    => AZ_TOGGLE,
//                UIBtnMode::ValueDrag => AZ_COARSE_DRAG,
//                UIBtnMode::ModTarget => AZ_MOD_SELECT,
//            };
//        let z1 = ActiveZone::from_rect(x, y, sub_type, (0.0, 0.0, size.0, size.1));
//        (f)(z1);
    }

    fn draw_value(&self, cr: &cairo::Context, x: f64, y: f64,
                  highlight: HLStyle, data: &dyn UIElementData,
                  value: f64, val_s: &str) {

        let (w, h) = self.size();
        let w = w - 2.0 * UI_GRPH_BORDER;
        let h = h - 2.0 * UI_GRPH_BORDER;
        let (xo, yo) = (UI_GRPH_BORDER, UI_GRPH_BORDER);

        cr.set_line_width(1.0);
        cr.set_source_rgb(
            UI_BTN_TXT_CLR.0,
            UI_BTN_TXT_CLR.1,
            UI_BTN_TXT_CLR.2);

        let data = data.as_graph_data().unwrap().data.borrow();

        cr.save();
        let mut first = true;
        for p in data.iter() {
            let px = 1.0 - (p.0 as f64);
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

//        let name = &data.as_btn_data().unwrap().label;
//        cr.set_font_size(UI_KNOB_FONT_SIZE);
//        cr.set_source_rgb(
//            UI_BTN_TXT_CLR.0,
//            UI_BTN_TXT_CLR.1,
//            UI_BTN_TXT_CLR.2);
//
//        let (xo, yo) = (
//            (UI_BTN_BORDER_WIDTH / 2.0).round(),
//            (UI_BTN_BORDER_WIDTH / 2.0).round(),
//        );
//
//        let w = UI_BTN_WIDTH;
//        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;
//
//        draw_centered_text(
//            cr,
//            x + xo,
//            y + yo + UI_ELEM_TXT_H + UI_BTN_BORDER_WIDTH,
//            w, (h / 2.0).round(), name);
//
//        match highlight {
//            HLStyle::Hover => {
//                cr.set_source_rgb(
//                    UI_BTN_TXT_HOVER_CLR.0,
//                    UI_BTN_TXT_HOVER_CLR.1,
//                    UI_BTN_TXT_HOVER_CLR.2);
//            },
//            HLStyle::HoverModTarget => {
//                cr.set_source_rgb(
//                    UI_BTN_TXT_HLHOVR_CLR.0,
//                    UI_BTN_TXT_HLHOVR_CLR.1,
//                    UI_BTN_TXT_HLHOVR_CLR.2);
//
//                cr.set_line_width(UI_BTN_BORDER_WIDTH);
//                self.define_border_path(cr, x + xo, y + yo, w, h);
//                cr.close_path();
//                cr.stroke();
//            },
//            HLStyle::ModTarget => {
//                cr.set_source_rgb(
//                    UI_BTN_TXT_HLIGHT_CLR.0,
//                    UI_BTN_TXT_HLIGHT_CLR.1,
//                    UI_BTN_TXT_HLIGHT_CLR.2);
//
//                cr.set_line_width(UI_BTN_BORDER2_WIDTH);
//                self.define_border_path(cr, x + xo, y + yo, w, h);
//                cr.close_path();
//                cr.stroke();
//            },
//            _ => { },
//        }
//
//        draw_centered_text(cr, x + xo, y + yo, w, (h / 2.0).round(), val_s);
    }

    fn draw_bg(&self, cr: &cairo::Context) {
        let (w, h) = self.size();

//        let w = w - 2.0 * UI_GRPH_BORDER;
//        let h = h - 2.0 * UI_GRPH_BORDER;
//        let (xo, yo) = (UI_GRPH_BORDER, UI_GRPH_BORDER);

        cr.set_source_rgb(
            UI_BTN_BG_CLR.0,
            UI_BTN_BG_CLR.1,
            UI_BTN_BG_CLR.2);
        cr.rectangle(0.0, 0.0, w, h);
        cr.fill();

        cr.set_line_width(UI_GRPH_BORDER);
        cr.set_source_rgb(
            UI_GRPH_BORDER_CLR.0,
            UI_GRPH_BORDER_CLR.1,
            UI_GRPH_BORDER_CLR.2);
        let mid_border = UI_GRPH_BORDER / 2.0;
        cr.rectangle(mid_border, mid_border, w - UI_GRPH_BORDER, h - UI_GRPH_BORDER);
        cr.stroke();
//        let (xo, yo) = (
//            (UI_BTN_BORDER_WIDTH / 2.0).round(),
//            (UI_BTN_BORDER_WIDTH / 2.0).round(),
//        );
//
//        let x = xo;
//        let y = yo;
//
//        let w = UI_BTN_WIDTH;
//        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;
//
//        println!("BUTON {},{}",x, y);
//
//
//        cr.set_line_width(UI_BTN_BORDER_WIDTH);
//        cr.set_source_rgb(
//            UI_BTN_BORDER_CLR.0,
//            UI_BTN_BORDER_CLR.1,
//            UI_BTN_BORDER_CLR.2);
//
//        // border
//        self.define_border_path(cr, x, y, w, h);
//        cr.stroke();
//
//        cr.set_line_width(UI_BTN_BORDER2_WIDTH);
//        cr.set_source_rgb(
//            UI_BTN_BORDER2_CLR.0,
//            UI_BTN_BORDER2_CLR.1,
//            UI_BTN_BORDER2_CLR.2);
//
//        self.define_border_path(cr, x, y, w, h);
//        cr.close_path();
//        cr.stroke();
//
//        cr.set_source_rgb(
//            UI_BTN_BG_CLR.0,
//            UI_BTN_BG_CLR.1,
//            UI_BTN_BG_CLR.2);
//
//        self.define_border_path(cr, x, y, w, h);
//        cr.fill();
//
//        // divider
//        cr.set_line_width(UI_BTN_BORDER2_WIDTH);
//        cr.set_source_rgb(
//            UI_BTN_BORDER2_CLR.0,
//            UI_BTN_BORDER2_CLR.1,
//            UI_BTN_BORDER2_CLR.2);
//
//        cr.move_to(x,     y + (h / 2.0).round());
//        cr.line_to(x + w, y + (h / 2.0).round());
//        cr.stroke();
    }
}
