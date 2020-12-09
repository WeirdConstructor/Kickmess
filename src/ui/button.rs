use crate::ui::painting::*;
use crate::ui::element::{UIElement, UIElementData};
use crate::ui::constants::*;
use crate::ui::util::{draw_centered_text};

pub struct Button {
}

impl Button {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl UIElement for Button {
    fn size(&self) -> (f64, f64) {
        (UI_BTN_WIDTH
         + UI_BTN_BORDER_WIDTH + UI_SAFETY_PAD,
         UI_ELEM_TXT_H + UI_BTN_BORDER_WIDTH + UI_ELEM_TXT_H
         + UI_BTN_BORDER_WIDTH + UI_SAFETY_PAD)
    }

    fn define_active_zones(&self, x: f64, y: f64, f: &mut dyn FnMut(ActiveZone)) {
        let size = self.size();
        let z1 = ActiveZone::from_rect(x, y, 0, (0.0, 0.0, size.0, size.1));
        (f)(z1);
    }

    fn draw_value(&self, cr: &cairo::Context, x: f64, y: f64,
                  hover_style: bool, data: &dyn UIElementData,
                  value: f64, val_s: &str) {

        let name = &data.as_btn_data().unwrap().label;
        cr.set_font_size(UI_KNOB_FONT_SIZE);
        cr.set_source_rgb(
            UI_BTN_TXT_CLR.0,
            UI_BTN_TXT_CLR.1,
            UI_BTN_TXT_CLR.2);

        let (xo, yo) = (
            (UI_BTN_BORDER_WIDTH / 2.0).round(),
            (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let w = UI_BTN_WIDTH;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        draw_centered_text(
            cr,
            x + xo,
            y + yo + UI_ELEM_TXT_H + UI_BTN_BORDER_WIDTH,
            w, (h / 2.0).round(), name);

        if hover_style {
            cr.set_source_rgb(
                UI_BTN_TXT_HOVER_CLR.0,
                UI_BTN_TXT_HOVER_CLR.1,
                UI_BTN_TXT_HOVER_CLR.2);
        }

        draw_centered_text(cr, x + xo, y + yo, w, (h / 2.0).round(), val_s);
    }

    fn draw_bg(&self, cr: &cairo::Context) {
        let (w, h) = self.size();

        let (xo, yo) = (
            (UI_BTN_BORDER_WIDTH / 2.0).round(),
            (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let x = xo;
        let y = yo;

        let w = UI_BTN_WIDTH;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        println!("BUTON {},{}",x, y);


        cr.set_line_width(UI_BTN_BORDER_WIDTH);
        cr.set_source_rgb(
            UI_BTN_BORDER_CLR.0,
            UI_BTN_BORDER_CLR.1,
            UI_BTN_BORDER_CLR.2);

        // border
        cr.move_to(x,                      y + UI_BTN_BEVEL);
        cr.line_to(x + UI_BTN_BEVEL,       y);
        cr.line_to(x + (w - UI_BTN_BEVEL), y);
        cr.line_to(x + w,                  y + UI_BTN_BEVEL);
        cr.line_to(x + w,                  y + (h - UI_BTN_BEVEL));
        cr.line_to(x + (w - UI_BTN_BEVEL), y + h);
        cr.line_to(x + UI_BTN_BEVEL,       y + h);
        cr.line_to(x,                      y + (h - UI_BTN_BEVEL));
        cr.close_path();
        cr.stroke();

        cr.set_line_width(UI_BTN_BORDER2_WIDTH);
        cr.set_source_rgb(
            UI_BTN_BORDER2_CLR.0,
            UI_BTN_BORDER2_CLR.1,
            UI_BTN_BORDER2_CLR.2);

        cr.move_to(x,                      y + UI_BTN_BEVEL);
        cr.line_to(x + UI_BTN_BEVEL,       y);
        cr.line_to(x + (w - UI_BTN_BEVEL), y);
        cr.line_to(x + w,                  y + UI_BTN_BEVEL);
        cr.line_to(x + w,                  y + (h - UI_BTN_BEVEL));
        cr.line_to(x + (w - UI_BTN_BEVEL), y + h);
        cr.line_to(x + UI_BTN_BEVEL,       y + h);
        cr.line_to(x,                      y + (h - UI_BTN_BEVEL));
        cr.close_path();

        cr.stroke();

        cr.set_source_rgb(
            UI_BTN_BG_CLR.0,
            UI_BTN_BG_CLR.1,
            UI_BTN_BG_CLR.2);

        cr.move_to(x,                      y + UI_BTN_BEVEL);
        cr.line_to(x + UI_BTN_BEVEL,       y);
        cr.line_to(x + (w - UI_BTN_BEVEL), y);
        cr.line_to(x + w,                  y + UI_BTN_BEVEL);
        cr.line_to(x + w,                  y + (h - UI_BTN_BEVEL));
        cr.line_to(x + (w - UI_BTN_BEVEL), y + h);
        cr.line_to(x + UI_BTN_BEVEL,       y + h);
        cr.line_to(x,                      y + (h - UI_BTN_BEVEL));
        cr.close_path();

        cr.fill();

        // divider
        cr.set_line_width(UI_BTN_BORDER2_WIDTH);
        cr.set_source_rgb(
            UI_BTN_BORDER2_CLR.0,
            UI_BTN_BORDER2_CLR.1,
            UI_BTN_BORDER2_CLR.2);

        cr.move_to(x,     y + (h / 2.0).round());
        cr.line_to(x + w, y + (h / 2.0).round());
        cr.stroke();
    }
}
