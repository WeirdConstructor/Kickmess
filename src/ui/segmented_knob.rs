use crate::ui::painting::*;
use crate::ui::element::{UIElement, UIElementData};
use crate::ui::constants::*;
use crate::ui::util::{draw_centered_text};

pub struct SegmentedKnob {
    sbottom:        (f64, f64),
    s:              [(f64, f64); 9],
    arc_len:        [f64; 7],
    full_len:       f64,
    s1_len:         f64,
    s2_len:         f64,
    radius:         f64,
    font_size_lbl:  f64,
    font_size_data: f64,
}

impl UIElement for SegmentedKnob {
    fn size(&self) -> (f64, f64) {
        let (lbl_x, lbl_y, lbl_w, lbl_h) = self.get_label_rect();

        (lbl_w + 2.0 * UI_SAFETY_PAD,
         (self.radius + lbl_y + lbl_h + 0.5 * UI_BG_KNOB_STROKE).round() + UI_SAFETY_PAD)
    }

    fn define_active_zones(&self, x: f64, y: f64, _elem_data: &dyn UIElementData, f: &mut dyn FnMut(ActiveZone)) {
        let (knob_xo, knob_yo) =
            self.get_center_offset(UI_BG_KNOB_STROKE);
        let (xo, yo) = (knob_xo, knob_yo);

        let z1 = ActiveZone::from_rect(x + xo, y + yo, AZ_COARSE_DRAG, self.get_coarse_adjustment_rect());
        (f)(z1);
        let z2 = ActiveZone::from_rect(x + xo, y + yo, AZ_FINE_DRAG, self.get_fine_adjustment_rect());
        (f)(z2);
    }

    fn draw_value(&self, cr: &cairo::Context, x: f64, y: f64,
                  highlight: HLStyle, data: &dyn UIElementData,
                  value: f64, val_s: &str) {

        let (knob_xo, knob_yo) =
            self.get_center_offset(UI_BG_KNOB_STROKE);
        let (xo, yo) = (knob_xo, knob_yo);

        match highlight {
            HLStyle::ModTarget => {
                self.draw_oct_arc(
                    &cr, x + xo, y + yo,
                    UI_MG_KNOB_STROKE,
                    UI_TXT_KNOB_HLIGHT_CLR,
                    false,
                    1.0);
            },
            HLStyle::HoverModTarget => {
                self.draw_oct_arc(
                    &cr, x + xo, y + yo,
                    UI_MG_KNOB_STROKE * 2.0,
                    UI_TXT_KNOB_HLHOVR_CLR,
                    false,
                    1.0);
            },
            HLStyle::Hover(subtype) => {
                if subtype == AZ_FINE_DRAG {
                    let r = self.get_fine_adjustment_mark();
                    cr.set_source_rgb(
                        UI_TXT_KNOB_HOVER_CLR.0,
                        UI_TXT_KNOB_HOVER_CLR.1,
                        UI_TXT_KNOB_HOVER_CLR.2);
                    cr.rectangle(x + xo + r.0, y + yo + r.1, r.2, r.3);
                    cr.fill();

                } else {
                    let r = self.get_coarse_adjustment_mark();
                    cr.set_source_rgb(
                        UI_TXT_KNOB_HOVER_CLR.0,
                        UI_TXT_KNOB_HOVER_CLR.1,
                        UI_TXT_KNOB_HOVER_CLR.2);
                    cr.rectangle(x + xo + r.0, y + yo + r.1, r.2, r.3);
                    cr.fill();
                }

                self.draw_oct_arc(
                    &cr, x + xo, y + yo,
                    UI_MG_KNOB_STROKE,
                    UI_FG_KNOB_STROKE_CLR,
                    true,
                    value);
            },
            HLStyle::None => {
                self.draw_oct_arc(
                    &cr, x + xo, y + yo,
                    UI_MG_KNOB_STROKE,
                    UI_FG_KNOB_STROKE_CLR,
                    true,
                    value);
            }
        }

        self.draw_value_label(&cr, x + xo, y + yo, highlight, val_s);

        let name = &data.as_knob_data().unwrap().label;
        self.draw_name(&cr, x + xo, y + yo, name);
    }

    fn draw_bg(&self, cr: &cairo::Context) {
        let (knob_xo, knob_yo) = self.get_center_offset(UI_BG_KNOB_STROKE);
        let (knob_w, knob_h)   = self.size();
        let (xo, yo) = (knob_xo, knob_yo);

        self.draw_oct_arc(
            &cr, xo, yo,
            UI_BG_KNOB_STROKE,
            UI_BG_KNOB_STROKE_CLR,
            false,
            1.0);

        cr.set_line_width(UI_BG_KNOB_STROKE);
        cr.set_source_rgb(
            UI_BG_KNOB_STROKE_CLR.0,
            UI_BG_KNOB_STROKE_CLR.1,
            UI_BG_KNOB_STROKE_CLR.2);

        let dc1 = self.get_decor_rect1();
        cr.rectangle(xo + dc1.0, yo + dc1.1, dc1.2, dc1.3);

        let valrect = self.get_value_rect();
        cr.rectangle(
            valrect.0 + xo, valrect.1 + yo, valrect.2, valrect.3);

        let lblrect = self.get_label_rect();
        cr.rectangle(
            lblrect.0 + xo, lblrect.1 + yo, lblrect.2, lblrect.3);
        cr.fill();

//        let r = self.get_coarse_adjustment_rect();
//        cr.set_source_rgb(0.0, 1.0, 1.0);
//        cr.set_line_width(1.0);
//        cr.rectangle(x + r.0, y + r.1, r.2, r.3);
//        cr.stroke();

        let r = self.get_coarse_adjustment_mark();
        cr.set_line_width(1.0);
        cr.rectangle(xo + r.0, yo + r.1, r.2, r.3);
        cr.fill();

//        let r = self.get_fine_adjustment_rect();
//        cr.set_source_rgb(1.0, 0.0, 1.0);
//        cr.set_line_width(1.0);
//        cr.rectangle(x + r.0, y + r.1, r.2, r.3);
//        cr.stroke();

        let r = self.get_fine_adjustment_mark();
        cr.rectangle(xo + r.0, yo + r.1, r.2, r.3);
        cr.fill();

        self.draw_oct_arc(
            &cr, xo, yo,
            UI_MG_KNOB_STROKE,
            UI_MG_KNOB_STROKE_CLR,
            false,
            1.0);
    }
}

impl SegmentedKnob {
    pub fn new(radius: f64, font_size_lbl: f64, font_size_data: f64) -> Self {
        let init_rot : f64 = 90.;
        // middle of the new surface
        let (xo, yo) =
            (radius + UI_BG_KNOB_STROKE * 2.0,
             radius + UI_BG_KNOB_STROKE * 2.0);

        let mut s       = [(0.0_f64, 0.0_f64); 9];
        let mut arc_len = [0.0_f64; 7];

        let sbottom = circle_point(radius, init_rot.to_radians());

        s[0] = circle_point(radius, (init_rot + 10.0_f64).to_radians());
        s[1] = circle_point(radius, (init_rot + 45.0_f64).to_radians());
        s[2] = circle_point(radius, (init_rot + 90.0_f64).to_radians());
        s[3] = circle_point(radius, (init_rot + 135.0_f64).to_radians());
        s[4] = circle_point(radius, (init_rot + 180.0_f64).to_radians());
        s[5] = circle_point(radius, (init_rot + 225.0_f64).to_radians());
        s[6] = circle_point(radius, (init_rot + 270.0_f64).to_radians());
        s[7] = circle_point(radius, (init_rot + 315.0_f64).to_radians());
        s[8] = circle_point(radius, (init_rot + 350.0_f64).to_radians());

        let s1_len  = ((s[0].0 - s[1].1).powf(2.0) + (s[0].0 - s[1].1).powf(2.0)).sqrt();
        let s2_len  = ((s[1].0 - s[2].1).powf(2.0) + (s[1].0 - s[2].1).powf(2.0)).sqrt();

        let full_len = s1_len * 2.0 + s2_len * 6.0;

        arc_len[0] = s1_len                  / full_len;
        arc_len[1] = (s1_len + s2_len)       / full_len;
        arc_len[2] = (s1_len + 2.0 * s2_len) / full_len;
        arc_len[3] = (s1_len + 3.0 * s2_len) / full_len;
        arc_len[4] = (s1_len + 4.0 * s2_len) / full_len;
        arc_len[5] = (s1_len + 5.0 * s2_len) / full_len;
        arc_len[6] = (s1_len + 6.0 * s2_len) / full_len;

        Self {
            sbottom,
            s,
            arc_len,
            full_len,
            s1_len,
            s2_len,
            radius,
            font_size_lbl,
            font_size_data,
        }
    }

    pub fn get_center_offset(&self, line_width: f64) -> (f64, f64) {
        ((self.get_label_rect().2 / 2.0).ceil() + UI_SAFETY_PAD,
         self.radius + (line_width / 2.0).ceil() + UI_SAFETY_PAD)
    }

    pub fn get_fine_adjustment_mark(&self) -> (f64, f64, f64, f64) {
        let r = self.get_fine_adjustment_rect();
        let mut size = (self.font_size_lbl * 0.25).round();
        if (size as i32) % 2 != 0 {
            size += 1.0;
        }
        (r.0 + (r.2 * 0.5 - size * 0.5).round(),
         r.1 + (r.3 * 0.5 + size * 0.5).round(),
         size,
         size)
    }

    pub fn get_fine_adjustment_rect(&self) -> (f64, f64, f64, f64) {
        let width = self.radius * 2.0;
        ((self.sbottom.0 - self.radius).round(),
         (self.sbottom.1 - self.radius).round(),
         width.round(),
         (self.radius).round())
    }

    pub fn get_coarse_adjustment_mark(&self) -> (f64, f64, f64, f64) {
        let r = self.get_coarse_adjustment_rect();
        let mut size = (self.font_size_lbl * 0.5).round();
        if (size as i32) % 2 != 0 {
            size += 1.0;
        }
        (r.0 + (r.2 * 0.5 - size * 0.5).round(),
         r.1 + (r.3 * 0.5 - size).round(),
         size,
         size)
    }

    pub fn get_coarse_adjustment_rect(&self) -> (f64, f64, f64, f64) {
        let width = self.radius * 2.0;
        ((self.sbottom.0 - self.radius).round(),
         -self.radius,
         width.round(),
         (self.radius).round())
    }

    pub fn get_value_rect(&self) -> (f64, f64, f64, f64) {
        let width = self.radius * 2.0;
        ((self.sbottom.0 - self.radius).round(),
         (self.sbottom.1 - (self.radius + UI_ELEM_TXT_H * 0.5)).round(),
         width.round(),
         UI_ELEM_TXT_H)
    }

    pub fn get_label_rect(&self) -> (f64, f64, f64, f64) {
        let width = self.radius * 2.5;
        ((self.sbottom.0 - width * 0.5).round(),
         (self.sbottom.1 + UI_BG_KNOB_STROKE).round(),
         width.round(),
         UI_ELEM_TXT_H)
    }

    pub fn get_decor_rect1(&self) -> (f64, f64, f64, f64) {
        ((self.s[0].0      - 0.25 * UI_BG_KNOB_STROKE).round(),
         (self.sbottom.1    - 0.5 * UI_BG_KNOB_STROKE).round(),
         ((self.s[8].0 - self.s[0].0).abs()
                           + 0.5 * UI_BG_KNOB_STROKE).round(),
         UI_BG_KNOB_STROKE * 3.0)
    }

    pub fn draw_name(&self, cr: &cairo::Context, x: f64, y: f64, s: &str) {
        let r = self.get_label_rect();
        cr.set_font_size(self.font_size_lbl);
        cr.set_source_rgb(
            UI_TXT_KNOB_CLR.0,
            UI_TXT_KNOB_CLR.1,
            UI_TXT_KNOB_CLR.2);

        draw_centered_text(cr, x + r.0, y + r.1, r.2, r.3, s);
    }

    pub fn draw_value_label(&self, cr: &cairo::Context, x: f64, y: f64, highlight: HLStyle, s: &str) {
        let r = self.get_value_rect();

        match highlight {
            HLStyle::Hover(_subtype) => {

                cr.set_font_size(self.font_size_data + 1.0);
                cr.set_source_rgb(
                    UI_TXT_KNOB_HOVER_CLR.0,
                    UI_TXT_KNOB_HOVER_CLR.1,
                    UI_TXT_KNOB_HOVER_CLR.2);
            },
            HLStyle::ModTarget => {
                cr.set_font_size(self.font_size_data + 1.0);
                cr.set_source_rgb(
                    UI_TXT_KNOB_HLIGHT_CLR.0,
                    UI_TXT_KNOB_HLIGHT_CLR.1,
                    UI_TXT_KNOB_HLIGHT_CLR.2);
            },
            _ => {
                cr.set_font_size(self.font_size_data);
            },
        }

        let ext = cr.text_extents(s);
        cr.move_to(
            x + r.0 + ((r.2 - ext.width)  / 2.0).abs().round(),
            y + r.1 + ext.height
                    + ((r.3 - ext.height) / 2.0).abs().round());
        cr.show_text(s);

    }

    pub fn draw_oct_arc(&self, cr: &cairo::Context, x: f64, y: f64, line_w: f64, color: (f64, f64, f64), with_dot: bool, value: f64) {
        cr.set_line_width(line_w);
        cr.set_source_rgb(color.0, color.1, color.2);
        let s       = &self.s;
        let arc_len = &self.arc_len;

        let (next_idx, segment_len, prev_arc_len) =
            if        value > self.arc_len[6] {
                (8, self.s1_len, self.arc_len[6])
            } else if value > self.arc_len[5] {
                (7, self.s2_len, self.arc_len[5])
            } else if value > self.arc_len[4] {
                (6, self.s2_len, self.arc_len[4])
            } else if value > self.arc_len[3] {
                (5, self.s2_len, self.arc_len[3])
            } else if value > self.arc_len[2] {
                (4, self.s2_len, self.arc_len[2])
            } else if value > self.arc_len[1] {
                (3, self.s2_len, self.arc_len[1])
            } else if value > self.arc_len[0] {
                (2, self.s2_len, self.arc_len[0])
            } else {
                (1, self.s1_len, 0.0)
            };

        cr.move_to(x + s[0].0, y + s[0].1);
        for i in 1..next_idx {
            cr.line_to(x + s[i].0, y + s[i].1);
        }

        let prev       = s[next_idx - 1];
        let last       = s[next_idx];
        let rest_len   = value - prev_arc_len;
        let rest_ratio = rest_len / (segment_len / self.full_len);
//        println!("i[{}] prev_arc_len={:1.3}, rest_len={:1.3}, value={:1.3}, seglen={:1.3}",
//                 next_idx, prev_arc_len, rest_len, value,
//                 segment_len / self.full_len);
        let partial =
            ((last.0 - prev.0) * rest_ratio,
             (last.1 - prev.1) * rest_ratio);

        cr.line_to(
            x + prev.0 + partial.0,
            y + prev.1 + partial.1);
        cr.stroke();

        if with_dot {
            cr.set_line_width(line_w * 0.5);
            cr.arc(
                x + prev.0 + partial.0,
                y + prev.1 + partial.1,
                line_w * 1.5, 0.0, 2.0 * std::f64::consts::PI);
        }

        cr.stroke();

    }
}

fn circle_point(r: f64, angle: f64) -> (f64, f64) {
    let (y, x) = angle.sin_cos();
    (x * r, y * r)
}
