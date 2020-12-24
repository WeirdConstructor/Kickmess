use crate::ui::painting::*;
use crate::ui::protocol::UIBtnMode;
use crate::ui::element::{UIElement, UIElementData};
use crate::ui::constants::*;

pub struct Button {
}

impl Button {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl Button {
    fn draw_border(&self, p: &dyn Painter, width: f64, clr: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, fill: bool) {
        let path = &[
            (x,                      y + UI_BTN_BEVEL),
            (x + UI_BTN_BEVEL,       y),
            (x + (w - UI_BTN_BEVEL), y),
            (x + w,                  y + UI_BTN_BEVEL),
            (x + w,                  y + (h - UI_BTN_BEVEL)),
            (x + (w - UI_BTN_BEVEL), y + h),
            (x + UI_BTN_BEVEL,       y + h),
            (x,                      y + (h - UI_BTN_BEVEL)),
        ];

        if fill {
            p.path_fill(clr, path);
        } else {
            p.path_stroke(width, clr, path);
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

    fn define_active_zones(&self, x: f64, y: f64, elem_data: &dyn UIElementData, f: &mut dyn FnMut(ActiveZone)) {
        let size     = self.size();
        let sub_type =
            match elem_data.as_btn_data().unwrap().mode {
                UIBtnMode::Toggle    => AZ_TOGGLE,
                UIBtnMode::ValueDrag => AZ_COARSE_DRAG,
                UIBtnMode::ModTarget => AZ_MOD_SELECT,
            };
        let z1 = ActiveZone::from_rect(x, y, sub_type, (0.0, 0.0, size.0, size.1));
        (f)(z1);
    }

    fn draw_value(&self, p: &dyn Painter, x: f64, y: f64,
                  highlight: HLStyle, data: &dyn UIElementData,
                  value: f64, val_s: &str) {

        let name = &data.as_btn_data().unwrap().label;

        let (xo, yo) = (
            (UI_BTN_BORDER_WIDTH / 2.0).round(),
            (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let w = UI_BTN_WIDTH;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        p.label(UI_KNOB_FONT_SIZE, 0, UI_BTN_TXT_CLR,
            x + xo,
            y + yo + UI_ELEM_TXT_H + UI_BTN_BORDER_WIDTH,
            w, (h / 2.0).round(), name);

        let color =
            match highlight {
                HLStyle::Hover(_) => UI_BTN_TXT_HOVER_CLR,
                HLStyle::HoverModTarget => {
                    self.draw_border(
                        p, UI_BTN_BORDER_WIDTH, UI_BTN_TXT_HLHOVR_CLR,
                        x + xo, y + yo, w, h, false);
                    UI_BTN_TXT_HLHOVR_CLR
                },
                HLStyle::ModTarget => {
                    self.draw_border(
                        p, UI_BTN_BORDER2_WIDTH, UI_BTN_TXT_HLIGHT_CLR,
                        x + xo, y + yo, w, h, false);
                    UI_BTN_TXT_HLIGHT_CLR
                },
                _ => UI_BTN_TXT_CLR,
            };

        p.label(UI_KNOB_FONT_SIZE, 0, color,
            x + xo, y + yo, w, (h / 2.0).round(), val_s);
    }

    fn draw_bg(&self, p: &dyn Painter, x: f64, y: f64) {
        let (w, h) = self.size();

        let (xo, yo) = (
            x + (UI_BTN_BORDER_WIDTH / 2.0).round(),
            y + (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let x = xo;
        let y = yo;

        let w = UI_BTN_WIDTH;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        println!("BUTON {},{}",x, y);

        // border
        self.draw_border(
            p, UI_BTN_BORDER_WIDTH, UI_BTN_BORDER_CLR, x, y, w, h, false);

        self.draw_border(
            p, UI_BTN_BORDER2_WIDTH, UI_BTN_BORDER2_CLR, x, y, w, h, false);

        self.draw_border(
            p, 0.0, UI_BTN_BG_CLR, x, y, w, h, true);

        // divider
        p.path_stroke(
            UI_BTN_BORDER2_WIDTH,
            UI_BTN_BORDER2_CLR,
            &[
                (x,     y + (h / 2.0).round()),
                (x + w, y + (h / 2.0).round()),
            ]);
    }
}
