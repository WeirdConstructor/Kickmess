use crate::KickmessVSTParams;
use vst::editor::Editor;
use std::sync::Arc;
use std::rc::Rc;

use pugl_sys::*;


struct KickmessUI {
    view:               PuglViewFFI,
    w:                  f64,
    h:                  f64,
    close_requested:    bool,
    zones:              Vec<ActiveZone>,
    draw_cache:         UIDrawCache,
    lbl_start:          Rc<String>,
    lbl_end:            Rc<String>,
    lbl_note:           Rc<String>,
}

impl KickmessUI {
    fn new(view: PuglViewFFI) -> Self {
        Self {
            view,
            w:               0.0,
            h:               0.0,
            close_requested: false,
            zones:           vec![],
            draw_cache:      UIDrawCache::new(),
            lbl_start:       Rc::new(String::from("Start")),
            lbl_end:         Rc::new(String::from("End")),
            lbl_note:        Rc::new(String::from("Note")),
        }
    }
}

enum ElementState {
    Active(f64),
    Hover(f64),
    Disabled(f64),
    TextEdit(String,u32),
}

enum Element {
    Knob(usize, Rc<String>),
    SmallKnob(usize,  Rc<String>),
    Toggle(usize, Rc<String>),
}

enum Connector {
    Down,
    Right,
}

struct ActiveZone {
    id:  usize,
    idx: u32,
    x:   u32,
    y:   u32,
    w:   u32,
    h:   u32,
}

trait WeirdUIDrawer {
    fn start_redraw(&mut self);
    fn done_redraw(&mut self);
    fn add_active_zone(&mut self, z: ActiveZone);
    fn paint_element_hbox(&mut self, name: &str, x: usize, y: usize, elements: &[Element], states: &[ElementState]);
}

const UI_BG_KNOB_STROKE       : f64 = 8.0;
const UI_MG_KNOB_STROKE       : f64 = 3.0;
const UI_FG_KNOB_STROKE       : f64 = 5.0;
const UI_MG_KNOB_STROKE_CLR   : (f64, f64, f64) = (0.26, 0.33, 0.57);
const UI_KNOB_RADIUS          : f64 = 30.0;
const UI_KNOB_SMALL_RADIUS    : f64 = 20.0;

const UI_BOX_H    : f64 = 70.0;
const UI_BOX_BORD : f64 =  3.0;
const UI_MARGIN   : f64 =  5.0;
const UI_PADDING  : f64 =  3.0;
const UI_ELEM_N_H : f64 = 65.0;
const UI_ELEM_N_W : f64 = 40.0;

struct SegmentedKnob {
    s0: (f64, f64),
    s1: (f64, f64),
    s2: (f64, f64),
    s3: (f64, f64),
    s4: (f64, f64),
    s5: (f64, f64),
    s6: (f64, f64),
    s7: (f64, f64),
    s8: (f64, f64),
    s1_arc_len: f64,
    s2_arc_len: f64,
    s3_arc_len: f64,
    s4_arc_len: f64,
    s5_arc_len: f64,
    s6_arc_len: f64,
    s7_arc_len: f64,
    s1_len: f64,
    s2_len: f64,
}

impl SegmentedKnob {
    fn new(radius: f64) -> Self {
        let init_rot = 90.;
        // middle of the new surface
        let (xo, yo) = (32., 32.);
        let s0 = circle_point(UI_KNOB_RADIUS, (init_rot + 10.0_f64).to_radians());
        let s1 = circle_point(UI_KNOB_RADIUS, (init_rot + 45.0_f64).to_radians());
        let s2 = circle_point(UI_KNOB_RADIUS, (init_rot + 90.0_f64).to_radians());
        let s3 = circle_point(UI_KNOB_RADIUS, (init_rot + 135.0_f64).to_radians());
        let s4 = circle_point(UI_KNOB_RADIUS, (init_rot + 180.0_f64).to_radians());
        let s5 = circle_point(UI_KNOB_RADIUS, (init_rot + 225.0_f64).to_radians());
        let s6 = circle_point(UI_KNOB_RADIUS, (init_rot + 270.0_f64).to_radians());
        let s7 = circle_point(UI_KNOB_RADIUS, (init_rot + 315.0_f64).to_radians());
        let s8 = circle_point(UI_KNOB_RADIUS, (init_rot + 350.0_f64).to_radians());

        let s1_len  = ((s0.0 - s1.1).powf(2.0) + (s0.0 - s1.1).powf(2.0)).sqrt();
        let s2_len  = ((s1.0 - s2.1).powf(2.0) + (s1.0 - s2.1).powf(2.0)).sqrt();

        let full_len = self.s0_len * 2.0 + self.s1_len * 6.0;

        Self {
            s0, s1, s2, s3, s4, s5, s6, s7, s8,
            s1_arc_len: s1_len                  / full_len,
            s2_arc_len: (s1_len + s2_len)       / full_len,
            s3_arc_len: (s1_len + 2.0 * s2_len) / full_len,
            s4_arc_len: (s1_len + 3.0 * s2_len) / full_len,
            s5_arc_len: (s1_len + 4.0 * s2_len) / full_len,
            s6_arc_len: (s1_len + 5.0 * s2_len) / full_len,
            s7_arc_len: (s1_len + 6.0 * s2_len) / full_len,
            s1_len,
            s2_len,
        }
    }

    fn draw_at_center(&self, cr: &cairo::Context, x, y, line_w: f64, color: (f64, f64, f64), arc_len: f64) {
        cr.set_line_width(line_w);
        cr.set_source_rgb(color.0, color.1, color.2);
        cr.move_to(x + self.s0.0, y + self.s0.1);
        if        arc_len > self.s1_arc_len {
        } else if arc_len > self.s2_arc_len {
        } else if arc_len > self.s3_arc_len {
        } else if arc_len > self.s4_arc_len {
        } else if arc_len > self.s5_arc_len {
        } else if arc_len > self.s6_arc_len {
        } else if arc_len > self.s7_arc_len {
        }

        cr.line_to(x + self.s1.0, y + self.s1.1);
        cr.line_to(x + self.s2.0, y + self.s2.1);
        cr.line_to(x + self.s3.0, y + self.s3.1);
        cr.line_to(x + self.s4.0, y + self.s4.1);
        cr.line_to(x + self.s5.0, y + self.s5.1);
        cr.line_to(x + self.s6.0, y + self.s6.1);
        cr.line_to(x + self.s7.0, y + self.s7.1);
        cr.line_to(x + self.s8.0, y + self.s8.1);
        cr.stroke();
    }
}

enum DrawCacheImg {
    Knob,
    KnobSmall,
}

impl UIDrawCache {
    fn new() -> Self {
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
        }
    }

    fn draw_knob(&mut self, cr: &cairo::Context, x: f64, y: f64) {
        if let None = self.surf[DrawCacheImg::Knob as usize] {
            let surf = cr.get_target().create_similar_image(
                cairo::Format::ARgb32,
                UI_ELEM_N_H as i32,
                UI_ELEM_N_H as i32).expect("Createable new img surface");
            self.surf[DrawCacheImg::Knob as usize] = Some(surf);

            cr.save();
            let init_rot = 90.;
            // middle of the new surface
            let (xo, yo) = (32., 32.);
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


            cr.set_line_width(UI_FG_KNOB_STROKE);
            cr.set_source_rgb(
                UI_MG_KNOB_STROKE_CLR.0,
                UI_MG_KNOB_STROKE_CLR.1,
                UI_MG_KNOB_STROKE_CLR.2);
            cr.move_to(xo + cx1, yo + cy1);
            cr.line_to(xo + cx2, yo + cy2);
            cr.line_to(xo + cx3, yo + cy3);
            cr.line_to(xo + cx4, yo + cy4);
            cr.line_to(xo + cx5, yo + cy5);
            cr.stroke();

            println!("LEN: {}", ((cx1 - cx2).powf(2.0) + (cy1 - cy2).powf(2.0)).sqrt());
            println!("LEN: {}", ((cx2 - cx3).powf(2.0) + (cy2 - cy3).powf(2.0)).sqrt());
            println!("LEN: {}", ((cx8 - cx9).powf(2.0) + (cy8 - cy9).powf(2.0)).sqrt());
        }

        let surf = &self.surf[DrawCacheImg::Knob as usize].as_ref().unwrap();

        cr.save();
        cr.set_source_surface(surf, x, y);
        cr.paint();
        cr.restore();
    }
}

struct UIDrawCache {
    surf: Vec<Option<cairo::Surface>>,
    knob_element_norm_len:    f64,
    knob_element_short_len:   f64,
    knob_s_element_norm_len:  f64,
    knob_s_element_short_len: f64,
}

struct CairoDrawer<'a, 'b, 'c> {
    cache:  &'c mut UIDrawCache,
    cr:     &'a cairo::Context,
    zones:  &'b mut Vec<ActiveZone>,
}

fn circle_point(r: f64, angle: f64) -> (f64, f64) {
    let (y, x) = angle.sin_cos();
    (x * r, y * r)
}

impl<'a, 'b, 'c> WeirdUIDrawer for CairoDrawer<'a, 'b, 'c> {
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

        self.cache.draw_knob(self.cr, 100., 100.);
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

impl PuglViewTrait for KickmessUI {
    fn exposed(&mut self, expose: &ExposeArea, cr: &cairo::Context) {
//        cr.set_source_rgb(0.2, 1.0, 0.2);
//        cr.rectangle(0., 0., 400., 400.);
//        cr.fill();
//
//        cr.save();
//        cr.set_source_rgb(0.9, 0.9, 0.9);
//        cr.arc(100., 100., 60.0, 0.0, 4.0);
//        cr.stroke();
//        cr.restore();

        let mut wd = CairoDrawer {
            cache: &mut self.draw_cache,
            cr,
            zones: &mut self.zones,
        };

        let elems = [
            Element::Knob(     0, self.lbl_start.clone()),
            Element::SmallKnob(1, self.lbl_end.clone()),
            Element::Toggle(   1, self.lbl_note.clone()),
        ];

        let states = [
            ElementState::Active(0.3),
            ElementState::Disabled(0.5),
            ElementState::Hover(0.5),
        ];

        wd.start_redraw();
        wd.paint_element_hbox("Frequency", 0, 0, &elems, &states);
        wd.done_redraw();
    }

    fn event(&mut self, ev: Event) -> Status {
//        println!("EVENT!");

        match ev.data {
            EventType::MouseMove(_) => {
                let pos = ev.pos();
                println!("MOUSEMOVE: {}:{}", pos.x, pos.y);
                self.post_redisplay();
            },
            _ => {},
        }

        Status::Success
    }

    fn resize(&mut self, size: Size) {
        println!("RESIZE {:?}", size);
//        self.w = size.w;
//        self.h = size.h;
//        self.post_redisplay();
    }

    fn close_request(&mut self) {
        println!("CLOSE REQ");
//        self.close_requested = true;
    }

    fn view(&self) -> PuglViewFFI {
        self.view
    }
}

pub(crate) struct KickmessEditor {
    view:      Option<Box<PuglView<KickmessUI>>>,
    params:    Arc<KickmessVSTParams>,
}

impl KickmessEditor {
    pub(crate) fn new(params: Arc<KickmessVSTParams>) -> Self {
        Self {
            view: None,
            params,
        }
    }
}

impl Editor for KickmessEditor {
    fn size(&self) -> (i32, i32) {
//        let hdl = self.view.as_ref().unwrap().as_ref().handle();
        (900, 400)
//        (hdl.get_frame().size.w as i32,
//         hdl.get_frame().size.h as i32)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
//        let hdl = self.view.as_ref().unwrap().as_ref().handle();
//        (hdl.get_frame().pos.x as i32,
//         hdl.get_frame().pos.y as i32)
    }

    fn open(&mut self, parent: *mut std::ffi::c_void) -> bool {
        println!("OPEN null={} == {:?}", parent.is_null(), parent);
        let mut view =
            PuglView::<KickmessUI>::new(
                parent,
                |pv| KickmessUI::new(pv));

        let ui = view.handle();
        ui.set_frame(Rect {
            pos: Coord { x: 0., y: 0. },
            size: Size { w: 900., h: 900. },
        });
        println!("title: {:?}", ui.set_window_title("Kickmess"));
        ui.make_resizable();
        println!("set_default_size: {:?}", ui.set_default_size(900, 400));
//        println!("show_window: {:?}", ui.realize());
        println!("show_window: {:?}", ui.show_window());

        self.view = Some(view);

        println!("OPENED");

        true
    }

    fn is_open(&mut self) -> bool {
        self.view.is_some()
    }

    fn idle(&mut self) {
        if let Some(view) = self.view.as_mut() {
            let hdl = view.as_mut().handle();

    //        println!("IDLE!");
            hdl.update(0.01);
//            hdl.draw();
    //        println!("IDLE!?!");

            if hdl.close_requested {
                println!("CLOSE REQ");
                self.view = None;
            }
        }
    }

    fn close(&mut self) {
        self.view.as_mut().unwrap().as_mut().handle().close_request()
    }
}
