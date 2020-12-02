use crate::ui::painting::*;
use crate::ui::constants::*;
use crate::ui::segmented_knob::*;

enum DrawCacheImg {
    Knob,
    KnobSmall,
}

pub struct DrawCache {
    surf:   Vec<Option<cairo::Surface>>,
    knob:   SegmentedKnob,
    knob_s: SegmentedKnob,
}

impl DrawCache {
    pub fn new() -> Self {
        Self {
            surf:   vec![None, None],
            knob:   SegmentedKnob::new(UI_KNOB_RADIUS),
            knob_s: SegmentedKnob::new(UI_KNOB_RADIUS * 0.8),
        }
    }

    pub fn draw_knob_data(&mut self, cr: &cairo::Context, x: f64, y: f64, hover_style: bool, value: f64, val_s: &str) {
        let (xo, yo) =
            ((UI_ELEM_N_H / 2.0).round(),
             (UI_ELEM_N_H / 2.0).round());

        self.knob.draw_oct_arc(
            &cr, x + xo, y + yo,
            UI_MG_KNOB_STROKE,
            UI_FG_KNOB_STROKE_CLR,
            true,
            value);

        self.knob.draw_value(&cr, x + xo, y + yo, hover_style, val_s);
    }

    pub fn draw_knob_bg(&mut self, cr: &cairo::Context, x: f64, y: f64, name: &str) -> ActiveZone {
        let (xo, yo) =
            ((UI_ELEM_N_H / 2.0).round(),
             (UI_ELEM_N_H / 2.0).round());

        if let None = self.surf[DrawCacheImg::Knob as usize] {
            let surf = cr.get_target().create_similar_image(
                cairo::Format::ARgb32,
                UI_ELEM_N_H as i32,
                UI_ELEM_N_H as i32).expect("Createable new img surface");
            self.surf[DrawCacheImg::Knob as usize] = Some(surf);
            let cr =
                cairo::Context::new(
                    self.surf[DrawCacheImg::Knob as usize].as_mut().unwrap());

            self.knob.draw_oct_arc(
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

            let dc1 = self.knob.get_decor_rect1();
            cr.rectangle(xo + dc1.0, yo + dc1.1, dc1.2, dc1.3);

            let valrect = self.knob.get_value_rect();
            cr.rectangle(
                valrect.0 + xo, valrect.1 + yo, valrect.2, valrect.3);

            let lblrect = self.knob.get_label_rect();
            cr.rectangle(
                lblrect.0 + xo, lblrect.1 + yo, lblrect.2, lblrect.3);
            cr.fill();

            self.knob.draw_oct_arc(
                &cr, xo, yo,
                UI_MG_KNOB_STROKE,
                UI_MG_KNOB_STROKE_CLR,
                false,
                1.0);

            self.knob.draw_name_bg(&cr, xo, yo, name);
        }

        let surf = &self.surf[DrawCacheImg::Knob as usize].as_ref().unwrap();

        cr.save();
        cr.set_source_surface(surf, x, y);
        cr.paint();
        cr.restore();

        ActiveZone::from_rect(x + xo, y + yo, 0, self.knob.get_value_rect())
    }
}

