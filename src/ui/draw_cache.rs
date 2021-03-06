// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use crate::ui::painting::*;
use crate::ui::constants::*;
use crate::ui::element::{UIElement, UIElementData};

pub struct DrawCache {
    elements: Vec<Box<dyn UIElement>>,
}

impl DrawCache {
    pub fn new() -> Self {
        Self {
            elements:   vec![],
        }
    }

    pub fn draw_container_label(&mut self, p: &mut dyn Painter, x: f64, y: f64, w: f64, lbl: &str) {
        p.rect_fill(UI_LBL_BG_CLR, x, y, w, UI_ELEM_TXT_H);
        p.path_stroke(
            UI_BORDER_WIDTH * 0.5,
            UI_BORDER_CLR,
            &mut ([
                (x, (y + UI_ELEM_TXT_H).round() + 0.5),
                (x + w, (y + UI_ELEM_TXT_H).round() + 0.5)
            ].iter().copied()), false);
        p.label(UI_CONT_FONT_SIZE, 0, UI_CONT_FONT_CLR, UI_SAFETY_PAD * 2.0 + x, y, w , UI_ELEM_TXT_H, lbl);
    }

    pub fn push_element(&mut self, el: Box<dyn UIElement>) {
//        self.surf.push(None);
        self.elements.push(el);
    }

    pub fn size_of(&self, idx: usize) -> (f64, f64) {
        if let Some(el) = self.elements.get(idx) {
            el.size()
        } else {
            (42.0, 42.0)
        }
    }

    pub fn draw_data(&mut self, p: &mut dyn Painter, x: f64, y: f64,
                     idx: usize, highlight: HLStyle,
                     data: &dyn UIElementData, value: f64, val_s: &str) {
        if let Some(el) = self.elements.get(idx) {
            el.draw_value(p, x, y, highlight, data, value, val_s);
        }
    }

    pub fn define_active_zones(&self, x: f64, y: f64, elem_data: &dyn UIElementData,
                               idx: usize, f: &mut dyn FnMut(ActiveZone)) {

        if let Some(el) = self.elements.get(idx) {
            el.define_active_zones(x, y, elem_data, f);
        }
    }

    pub fn draw_bg(&mut self, p: &mut dyn Painter, x: f64, y: f64, idx: usize) {
        if let Some(el) = self.elements.get(idx) {
            el.draw_bg(p, x, y);
        }
    }
}

