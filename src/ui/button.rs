// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use crate::ui::painting::*;
use crate::ui::protocol::UIBtnMode;
use crate::ui::element::{UIElement, UIElementData};
use crate::ui::constants::*;

pub struct Button {
    width:      f64,
    font_size:  f64,
}

impl Button {
    pub fn new(width: f64, font_size: f64) -> Self {
        Self {
            width,
            font_size,
        }
    }
}

impl Button {
    fn draw_border(&self, p: &mut dyn Painter, width: f64, clr: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, fill: bool) {
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
            p.path_fill(clr, &mut path.iter().copied(), true);
        } else {
            p.path_stroke(width, clr, &mut path.iter().copied(), true);
        }
    }
}

impl Button {
    fn draw_divider(&self, p: &mut dyn Painter, width: f64, color: (f64, f64, f64), x: f64, y: f64) {
        let (x, y) = (
            x + (UI_BTN_BORDER_WIDTH / 2.0).round(),
            y + (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let w = self.width;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        // divider
        p.path_stroke(
            UI_BTN_BORDER2_WIDTH,
            color,
            &mut [
                (x,     y + (h / 2.0).round()),
                (x + w, y + (h / 2.0).round()),
            ].iter().copied(),
            false);
    }

}

impl UIElement for Button {
    fn size(&self) -> (f64, f64) {
        (self.width
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

    fn draw_value(&self, p: &mut dyn Painter, x: f64, y: f64,
                  highlight: HLStyle, data: &dyn UIElementData,
                  value: f64, val_s: &str) {

        let name = &data.as_btn_data().unwrap().label;

        let (xo, yo) = (
            (UI_BTN_BORDER_WIDTH / 2.0).round(),
            (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let w = self.width;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        p.label(self.font_size, 0, UI_BTN_TXT_CLR,
            x + xo,
            y + yo + UI_ELEM_TXT_H + UI_BTN_BORDER2_WIDTH,
            w, (h / 2.0).round(), name);

        let color =
            match highlight {
                HLStyle::Hover(_) => {
                    self.draw_border(
                        p, UI_BTN_BORDER2_WIDTH, UI_BTN_TXT_HOVER_CLR,
                        x + xo - (UI_BTN_BORDER2_WIDTH * 0.5).round(),
                        y + yo - (UI_BTN_BORDER2_WIDTH * 0.5).round(),
                        w + UI_BTN_BORDER2_WIDTH,
                        h + UI_BTN_BORDER2_WIDTH, false);
                    UI_BTN_TXT_HOVER_CLR
                },
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
                HLStyle::Inactive => {
                    self.draw_border(
                        p, UI_BTN_BORDER2_WIDTH, UI_INACTIVE_CLR,
                        x + xo, y + yo, w, h, false);
                    self.draw_divider(
                        p, UI_BTN_BORDER2_WIDTH * 1.2, UI_INACTIVE_CLR, x, y);
                    UI_INACTIVE2_CLR
                },
                _ => UI_BTN_TXT_CLR,
            };

        p.label(self.font_size, 0, color,
            x + xo, y + yo, w, (h / 2.0).round(), val_s);
    }

    fn draw_bg(&self, p: &mut dyn Painter, x: f64, y: f64) {
        let (xo, yo) = (
            x + (UI_BTN_BORDER_WIDTH / 2.0).round(),
            y + (UI_BTN_BORDER_WIDTH / 2.0).round(),
        );

        let w = self.width;
        let h = UI_ELEM_TXT_H * 2.0 + UI_BTN_BORDER_WIDTH;

        // border
        self.draw_border(
            p, UI_BTN_BORDER_WIDTH, UI_BTN_BORDER_CLR, xo, yo, w, h, false);

        self.draw_border(
            p, UI_BTN_BORDER2_WIDTH, UI_BTN_BORDER2_CLR, xo, yo, w, h, false);

        self.draw_border(
            p, 0.0, UI_BTN_BG_CLR, xo, yo, w, h, true);

        self.draw_divider(p, UI_BTN_BORDER2_WIDTH, UI_BTN_BORDER2_CLR, x, y);
    }
}
