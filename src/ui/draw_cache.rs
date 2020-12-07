use crate::ui::painting::*;
use crate::ui::constants::*;
use crate::ui::element::*;
use crate::ui::segmented_knob::*;

#[derive(Debug, Clone, Copy)]
pub enum DrawCacheImg {
    Knob,
    KnobSmall,
    KnobHuge,
//    Button,
}

pub struct DrawCache {
    surf:   Vec<Option<cairo::Surface>>,
    knob:   SegmentedKnob,
    knob_s: SegmentedKnob,
    knob_h: SegmentedKnob,
//    button: SegmentedButton,
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
//            button: SegmentedButton::new(UI_KNOB_FONT_SIZE),
        }
    }

    pub fn size_of(&self, size: DrawCacheImg) -> (f64, f64) {
        match size {
            DrawCacheImg::Knob      => self.knob.size(UI_BG_KNOB_STROKE),
            DrawCacheImg::KnobSmall => self.knob_s.size(UI_BG_KNOB_STROKE),
            DrawCacheImg::KnobHuge  => self.knob_h.size(UI_BG_KNOB_STROKE),
//            DrawCacheImg::Button    => self.button.size(UI_BG_KNOB_STROKE),
        }
    }

    pub fn draw_data(&mut self, cr: &cairo::Context, x: f64, y: f64,
                     size: DrawCacheImg, hover_style: bool, name: &str, value: f64, val_s: &str) {
        let knob =
            match size {
                DrawCacheImg::Knob      => &self.knob,
                DrawCacheImg::KnobSmall => &self.knob_s,
                DrawCacheImg::KnobHuge  => &self.knob_h,
            };

        knob.draw_value(cr, x, y, hover_style, name, value, val_s);
    }

    pub fn define_active_zones(&self, x: f64, y: f64, size: DrawCacheImg, f: &mut dyn FnMut(ActiveZone)) {
        let knob =
            match size {
                DrawCacheImg::Knob      => &self.knob,
                DrawCacheImg::KnobSmall => &self.knob_s,
                DrawCacheImg::KnobHuge  => &self.knob_h,
            };
        knob.define_active_zones(x, y, f);
    }

    pub fn draw_bg(&mut self, cr: &cairo::Context, x: f64, y: f64, size: DrawCacheImg) {
        let knob =
            match size {
                DrawCacheImg::Knob      => &self.knob,
                DrawCacheImg::KnobSmall => &self.knob_s,
                DrawCacheImg::KnobHuge  => &self.knob_h,
            };
        let knob_surface_idx = size as usize;
        let (knob_w, knob_h) = knob.size(UI_BG_KNOB_STROKE);

        if let None = self.surf[knob_surface_idx] {
            let surf = cr.get_target().create_similar_image(
                cairo::Format::ARgb32,
                knob_w as i32,
                knob_h as i32).expect("Createable new img surface");
            self.surf[knob_surface_idx] = Some(surf);
            let cr =
                cairo::Context::new(
                    self.surf[knob_surface_idx].as_mut().unwrap());
            knob.draw_bg(&cr, x, y);
        }

        let surf = &self.surf[knob_surface_idx].as_ref().unwrap();

        cr.save();
        cr.set_source_surface(surf, x, y);
        cr.paint();
    }
}

