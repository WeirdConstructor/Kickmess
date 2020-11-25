
pub trait PlugUI : Send {
    fn get_label(&mut self, idx: usize) -> String;
    fn needs_redraw(&mut self) -> bool;
    fn redraw(&mut self, state: &mut PlugUIPainter);
    fn handle_input(&mut self, state: &mut PlugUIPainter);
}

pub struct PlugUIState {
    zones: Vec<ActiveZone>,
    cache: UIDrawCache,
}

impl PlugUIState {
    pub fn new() -> Self {
        Self {
            zones: vec![],
            cache: UIDrawCache::new(),
        }
    }
}

pub enum ElementState {
    Active(f64),
    Hover(f64),
    Disabled(f64),
    TextEdit(String,u32),
}

pub enum Element {
    //   id,            label_idx
    Knob(usize,         usize),
    SmallKnob(usize,    usize),
    Toggle(usize,       usize),
}

enum Connector {
    Down,
    Right,
}

pub struct ActiveZone {
    id:  usize,
    idx: u32,
    x:   u32,
    y:   u32,
    w:   u32,
    h:   u32,
}

pub trait UIPainter {
    fn start_redraw(&mut self);
    fn done_redraw(&mut self);
    fn add_active_zone(&mut self, z: ActiveZone);
    fn paint_element_hbox(&mut self, name: &str, x: usize, y: usize, elements: &[Element], states: &[ElementState]);
}

const UI_BG_KNOB_STROKE       : f64 = 8.0;
const UI_MG_KNOB_STROKE       : f64 = 3.0;
const UI_FG_KNOB_STROKE       : f64 = 5.0;
const UI_MG_KNOB_STROKE_CLR   : (f64, f64, f64) = (0.26, 0.33, 0.57);
const UI_FG_KNOB_STROKE_CLR   : (f64, f64, f64) = (0.84, 0.76, 0.32);
const UI_KNOB_RADIUS          : f64 = 30.0;
const UI_KNOB_SMALL_RADIUS    : f64 = 20.0;

const UI_BOX_H      : f64 = 200.0;
const UI_BOX_BORD   : f64 =   3.0;
const UI_MARGIN     : f64 =   5.0;
const UI_PADDING    : f64 =   3.0;
const UI_ELEM_N_H   : f64 = 120.0;
const UI_ELEM_N_W   : f64 =  80.0;
const UI_ELEM_TXT_H : f64 =  20.0;

struct SegmentedKnob {
    sc:             (f64, f64),
    s:              [(f64, f64); 9],
    arc_len:        [f64; 7],
    full_len:       f64,
    s1_len:         f64,
    s2_len:         f64,
}

impl SegmentedKnob {
    fn new(radius: f64) -> Self {
        let init_rot : f64 = 90.;
        // middle of the new surface
        let (xo, yo) =
            (radius + UI_BG_KNOB_STROKE * 2.0,
             radius + UI_BG_KNOB_STROKE * 2.0);

        let mut s       = [(0.0_f64, 0.0_f64); 9];
        let mut arc_len = [0.0_f64; 7];

        let sc = circle_point(radius, init_rot.to_radians());

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
            sc,
            s,
            arc_len,
            full_len,
            s1_len,
            s2_len,
        }
    }

    fn get_value_rect(&self) -> (f64, f64, f64, f64) {
        (0.0, 0.0, 0.0, 0.0)
    }

    fn get_label_rect(&self) -> (f64, f64, f64, f64) {
        (0.0, 0.0, 0.0, 0.0)
    }

    fn get_label_connect_rect(&self) -> (f64, f64, f64, f64) {
        (0.0, 0.0, 0.0, 0.0)
    }

    fn draw_at_center(&self, cr: &cairo::Context, x: f64, y: f64, line_w: f64, color: (f64, f64, f64), value: f64) {
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

        let prev = s[next_idx - 1];
        let last = s[next_idx];
        let rest_len = value - prev_arc_len;
        println!("i[{}] prev_arc_len={:1.3}, rest_len={:1.3}, value={:1.3}, seglen={:1.3}",
                 next_idx, prev_arc_len, rest_len, value,
                 segment_len / self.full_len);
        let rest_ratio = rest_len / (segment_len / self.full_len);
        let partial =
            ((last.0 - prev.0) * rest_ratio,
             (last.1 - prev.1) * rest_ratio);

        cr.line_to(x + prev.0 + partial.0, y + prev.1 + partial.1);

        cr.stroke();
    }
}

enum DrawCacheImg {
    Knob,
    KnobSmall,
}

impl UIDrawCache {
    pub fn new() -> Self {
        // calculate the length of the knobs long and short
        // elements
        let init_rot = 90.;
        let (cx1, cy1) = circle_point(UI_KNOB_RADIUS, (init_rot + 10.0_f64).to_radians());
        let (cx2, cy2) = circle_point(UI_KNOB_RADIUS, (init_rot + 45.0_f64).to_radians());
        let (cx3, cy3) = circle_point(UI_KNOB_RADIUS, (init_rot + 90.0_f64).to_radians());
        let knob_element_norm_len  = ((cx1 - cx2).powf(2.0) + (cy1 - cy2).powf(2.0)).sqrt();
        let knob_element_short_len = ((cx2 - cx3).powf(2.0) + (cy2 - cy3).powf(2.0)).sqrt();

        let (cx1, cy1) = circle_point(UI_KNOB_SMALL_RADIUS, (init_rot + 10.0_f64).to_radians());
        let (cx2, cy2) = circle_point(UI_KNOB_SMALL_RADIUS, (init_rot + 45.0_f64).to_radians());
        let (cx3, cy3) = circle_point(UI_KNOB_SMALL_RADIUS, (init_rot + 90.0_f64).to_radians());
        let knob_s_element_norm_len  = ((cx1 - cx2).powf(2.0) + (cy1 - cy2).powf(2.0)).sqrt();
        let knob_s_element_short_len = ((cx2 - cx3).powf(2.0) + (cy2 - cy3).powf(2.0)).sqrt();

        Self {
            surf: vec![None, None],
            knob_element_norm_len,
            knob_element_short_len,
            knob_s_element_norm_len,
            knob_s_element_short_len,
            knob: SegmentedKnob::new(UI_KNOB_RADIUS),
        }
    }

    fn draw_knob(&mut self, cr: &cairo::Context, x: f64, y: f64) {
        let (xo, yo) =
            ((UI_ELEM_N_H / 2.0).round(),
             (UI_ELEM_N_H / 2.0).round());

        if let None = self.surf[DrawCacheImg::Knob as usize] {
            let surf = cr.get_target().create_similar_image(
                cairo::Format::ARgb32,
                UI_ELEM_N_H as i32,
                UI_ELEM_N_H as i32).expect("Createable new img surface");
            self.surf[DrawCacheImg::Knob as usize] = Some(surf);

            cr.save();
            let init_rot : f64 = 90.;
            // middle of the new surface

            let c_bottom   = circle_point(UI_KNOB_RADIUS, (init_rot).to_radians());

            let (cx1, cy1) = circle_point(UI_KNOB_RADIUS, (init_rot + 10.0_f64).to_radians());
            let (cx2, cy2) = circle_point(UI_KNOB_RADIUS, (init_rot + 45.0_f64).to_radians());
            let (cx3, cy3) = circle_point(UI_KNOB_RADIUS, (init_rot + 90.0_f64).to_radians());
            let (cx4, cy4) = circle_point(UI_KNOB_RADIUS, (init_rot + 135.0_f64).to_radians());
            let (cx5, cy5) = circle_point(UI_KNOB_RADIUS, (init_rot + 180.0_f64).to_radians());
            let (cx6, cy6) = circle_point(UI_KNOB_RADIUS, (init_rot + 225.0_f64).to_radians());
            let (cx7, cy7) = circle_point(UI_KNOB_RADIUS, (init_rot + 270.0_f64).to_radians());
            let (cx8, cy8) = circle_point(UI_KNOB_RADIUS, (init_rot + 315.0_f64).to_radians());
            let (cx9, cy9) = circle_point(UI_KNOB_RADIUS, (init_rot + 350.0_f64).to_radians());


            let cr = cairo::Context::new(self.surf[DrawCacheImg::Knob as usize].as_mut().unwrap());

            cr.set_line_width(UI_BG_KNOB_STROKE);
            cr.set_source_rgb(0.28, 0.28, 0.28);
            cr.move_to(xo + cx1, yo + cy1);
            cr.line_to(xo + cx2, yo + cy2);
            cr.line_to(xo + cx3, yo + cy3);
            cr.line_to(xo + cx4, yo + cy4);
            cr.line_to(xo + cx5, yo + cy5);
            cr.line_to(xo + cx6, yo + cy6);
            cr.line_to(xo + cx7, yo + cy7);
            cr.line_to(xo + cx8, yo + cy8);
            cr.line_to(xo + cx9, yo + cy9);
            cr.stroke();

            cr.rectangle(
                (xo + cx1) - 0.25 * UI_BG_KNOB_STROKE,
                (yo + c_bottom.1) - 0.5 * UI_BG_KNOB_STROKE,
                (cx9 - cx1).abs() + 0.5 * UI_BG_KNOB_STROKE,
                UI_BG_KNOB_STROKE * 3.0);
            cr.fill();

            cr.rectangle(
                (xo + c_bottom.0) - UI_ELEM_N_W * 0.5,
                (yo + c_bottom.1) + UI_BG_KNOB_STROKE,
                UI_ELEM_N_W,
                UI_ELEM_TXT_H);
            cr.fill();

            cr.set_line_width(UI_MG_KNOB_STROKE);
            cr.set_source_rgb(
                UI_MG_KNOB_STROKE_CLR.0,
                UI_MG_KNOB_STROKE_CLR.1,
                UI_MG_KNOB_STROKE_CLR.2);
            cr.move_to(xo + cx1, yo + cy1);
            cr.line_to(xo + cx2, yo + cy2);
            cr.line_to(xo + cx3, yo + cy3);
            cr.line_to(xo + cx4, yo + cy4);
            cr.line_to(xo + cx5, yo + cy5);
            cr.line_to(xo + cx6, yo + cy6);
            cr.line_to(xo + cx7, yo + cy7);
            cr.line_to(xo + cx8, yo + cy8);
            cr.line_to(xo + cx9, yo + cy9);
            cr.stroke();

//            cr.set_line_width(UI_FG_KNOB_STROKE);
//            cr.set_source_rgb(
//                UI_FG_KNOB_STROKE_CLR.0,
//                UI_FG_KNOB_STROKE_CLR.1,
//                UI_FG_KNOB_STROKE_CLR.2);
//            cr.move_to(xo + cx1, yo + cy1);
//            cr.line_to(xo + cx2, yo + cy2);
//            cr.line_to(xo + cx3, yo + cy3);
//            cr.line_to(xo + cx4, yo + cy4);
//            cr.line_to(xo + cx5, yo + cy5);
//            cr.stroke();

            println!("LEN: {}", ((cx1 - cx2).powf(2.0) + (cy1 - cy2).powf(2.0)).sqrt());
            println!("LEN: {}", ((cx2 - cx3).powf(2.0) + (cy2 - cy3).powf(2.0)).sqrt());
            println!("LEN: {}", ((cx8 - cx9).powf(2.0) + (cy8 - cy9).powf(2.0)).sqrt());
        }

        let surf = &self.surf[DrawCacheImg::Knob as usize].as_ref().unwrap();



        cr.save();
        cr.set_source_surface(surf, x, y);
        cr.paint();
        cr.restore();

        self.knob.draw_at_center(
            &cr, x + 10.0, y + 210.0,
            UI_MG_KNOB_STROKE,
            UI_FG_KNOB_STROKE_CLR,
            1.0);

        self.knob.draw_at_center(
            &cr, x + xo, y + yo,
            UI_MG_KNOB_STROKE,
            UI_FG_KNOB_STROKE_CLR,
            0.1);

        self.knob.draw_at_center(
            &cr, x + 90.0, y + 210.0,
            UI_MG_KNOB_STROKE,
            UI_FG_KNOB_STROKE_CLR,
            0.5);

        self.knob.draw_at_center(
            &cr, x + 190.0, y + 210.0,
            UI_MG_KNOB_STROKE,
            UI_FG_KNOB_STROKE_CLR,
            0.4);

        self.knob.draw_at_center(
            &cr, x + 10.0, y + 290.0,
            UI_MG_KNOB_STROKE,
            UI_FG_KNOB_STROKE_CLR,
            0.3);
    }
}

pub struct UIDrawCache {
    surf:                     Vec<Option<cairo::Surface>>,
    knob_element_norm_len:    f64,
    knob_element_short_len:   f64,
    knob_s_element_norm_len:  f64,
    knob_s_element_short_len: f64,
    knob:                     SegmentedKnob,
}

pub struct PlugUIPainter<'a, 'b> {
    cr:     &'b cairo::Context,
    zones:  &'a mut Vec<ActiveZone>,
    cache:  &'a mut UIDrawCache,
}

impl<'a, 'b> PlugUIPainter<'a, 'b> {
    pub fn new(uistate: &'a mut PlugUIState,
               cr: &'b cairo::Context) -> Self {

        Self {
            cr,
            zones: &mut uistate.zones,
            cache: &mut uistate.cache,
        }
    }
}

fn circle_point(r: f64, angle: f64) -> (f64, f64) {
    let (y, x) = angle.sin_cos();
    (x * r, y * r)
}

impl<'a, 'b> UIPainter for PlugUIPainter<'a, 'b> {
    fn paint_element_hbox(&mut self, name: &str, x: usize, y: usize, elements: &[Element], states: &[ElementState])
    {
        let mut w =
            elements.iter().fold(0.0, |w, e| {
                w + 2.0 * UI_PADDING + match e {
                    Element::Knob(_, _)      => UI_ELEM_N_W,
                    Element::SmallKnob(_, _) => UI_ELEM_N_W * 0.8,
                    Element::Toggle(_, _)    => UI_ELEM_N_W,
                }
            });


        let mut h = UI_BOX_H + 2.0 * UI_PADDING;

        let x = x as f64 * (UI_ELEM_N_W + UI_MARGIN);
        let y = y as f64 * (UI_ELEM_N_H + UI_MARGIN);

        self.cr.save();
        self.cr.set_line_width(1.0);

        self.cr.set_source_rgb(0.29, 0.29, 0.29);
        self.cr.rectangle(x, y, w, h);
        self.cr.fill();

        self.cr.set_source_rgb(0.54, 0.54, 0.54);
        self.cr.rectangle(
            x + UI_PADDING,
            y + UI_PADDING,
            w - 2.0 * UI_PADDING,
            h - 2.0 * UI_PADDING);
        self.cr.fill();
        self.cr.restore();

//        self.cache.draw_knob(self.cr, 10., 10.);
        self.cache.draw_knob(self.cr, 200., 100.);
    }

    fn start_redraw(&mut self)
    {
        self.zones.clear();
    }

    fn done_redraw(&mut self)
    {
    }

    fn add_active_zone(&mut self, z: ActiveZone)
    {
        self.zones.push(z);
    }
}

