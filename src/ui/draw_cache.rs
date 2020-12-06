use crate::ui::painting::*;
use crate::ui::constants::*;
use crate::ui::segmented_knob::*;

#[derive(Debug, Clone, Copy)]
pub enum DrawCacheImg {
    Knob,
    KnobSmall,
    KnobHuge,
}

pub struct DrawCache {
    surf:   Vec<Option<cairo::Surface>>,
    knob:   SegmentedKnob,
    knob_s: SegmentedKnob,
    knob_h: SegmentedKnob,
}

impl DrawCache {
    pub fn new() -> Self {
        Self {
            surf:   vec![None, None, None],
            knob_h: SegmentedKnob::new(
                        (UI_KNOB_RADIUS * 1.3).round(),
                        (UI_KNOB_FONT_SIZE + 2.0).round(),
                        UI_KNOB_FONT_SIZE + 1.0),
            knob:   SegmentedKnob::new(
                        UI_KNOB_RADIUS,
                        UI_KNOB_FONT_SIZE,
                        UI_KNOB_FONT_SIZE - 1.0),
            knob_s: SegmentedKnob::new(
                        (UI_KNOB_RADIUS * 0.75).round(),
                        (UI_KNOB_FONT_SIZE * 0.75).round(),
                        ((UI_KNOB_FONT_SIZE - 1.0) * 0.8).round()),
        }
    }

    pub fn size_of(&self, size: DrawCacheImg) -> (f64, f64) {
        match size {
            DrawCacheImg::Knob      => self.knob.size(UI_BG_KNOB_STROKE),
            DrawCacheImg::KnobSmall => self.knob_s.size(UI_BG_KNOB_STROKE),
            DrawCacheImg::KnobHuge  => self.knob_h.size(UI_BG_KNOB_STROKE),
        }
    }

    pub fn draw_data(&mut self, cr: &cairo::Context, x: f64, y: f64, size: DrawCacheImg, hover_style: bool, value: f64, val_s: &str) {
        let knob =
            match size {
                DrawCacheImg::Knob      => &self.knob,
                DrawCacheImg::KnobSmall => &self.knob_s,
                DrawCacheImg::KnobHuge  => &self.knob_h,
            };

        let (knob_xo, knob_yo) = knob.get_center_offset(UI_BG_KNOB_STROKE);
        let (xo, yo) = (knob_xo, knob_yo);

        knob.draw_oct_arc(
            &cr, x + xo, y + yo,
            UI_MG_KNOB_STROKE,
            UI_FG_KNOB_STROKE_CLR,
            true,
            value);

        knob.draw_value(&cr, x + xo, y + yo, hover_style, val_s);
    }

    pub fn draw_bg(&mut self, cr: &cairo::Context, x: f64, y: f64, size: DrawCacheImg, name: &str) -> ActiveZone {
        let knob =
            match size {
                DrawCacheImg::Knob      => &self.knob,
                DrawCacheImg::KnobSmall => &self.knob_s,
                DrawCacheImg::KnobHuge  => &self.knob_h,
            };
        let knob_surface_idx = size as usize;

        let (knob_xo, knob_yo) = knob.get_center_offset(UI_BG_KNOB_STROKE);
        let (knob_w, knob_h)   = knob.size(UI_BG_KNOB_STROKE);
        let (xo, yo) = (knob_xo, knob_yo);

        if let None = self.surf[knob_surface_idx] {
            let surf = cr.get_target().create_similar_image(
                cairo::Format::ARgb32,
                knob_w as i32,
                knob_h as i32).expect("Createable new img surface");
            self.surf[knob_surface_idx] = Some(surf);
            let cr =
                cairo::Context::new(
                    self.surf[knob_surface_idx].as_mut().unwrap());

            knob.draw_oct_arc(
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

            let dc1 = knob.get_decor_rect1();
            cr.rectangle(xo + dc1.0, yo + dc1.1, dc1.2, dc1.3);

            let valrect = knob.get_value_rect();
            cr.rectangle(
                valrect.0 + xo, valrect.1 + yo, valrect.2, valrect.3);

            let lblrect = knob.get_label_rect();
            cr.rectangle(
                lblrect.0 + xo, lblrect.1 + yo, lblrect.2, lblrect.3);
            cr.fill();

            knob.draw_oct_arc(
                &cr, xo, yo,
                UI_MG_KNOB_STROKE,
                UI_MG_KNOB_STROKE_CLR,
                false,
                1.0);
        }

        let surf = &self.surf[knob_surface_idx].as_ref().unwrap();

        cr.save();
        cr.set_source_surface(surf, x, y);
        cr.paint();
        knob.draw_name_bg(&cr, x + xo, y + yo, name);

        if false {
            cr.set_line_width(0.5);
            cr.set_source_rgb(1.0, 0.0, 1.0);
            cr.rectangle(x + xo, y + yo, 2.0, 2.0);
            cr.fill();

            cr.set_source_rgb(0.0, 1.0, 1.0);
            let s = knob.size(UI_BG_KNOB_STROKE);
            cr.rectangle(x, y, s.0, s.1);
            cr.stroke();
            cr.restore();
        }

        ActiveZone::from_rect(x + xo, y + yo, 0, knob.get_value_rect())
    }
}

