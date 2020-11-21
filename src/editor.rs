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

struct CairoDrawer<'a, 'b> {
    cr:     &'a cairo::Context,
    zones:  &'b mut Vec<ActiveZone>,
}

const UI_BOX_H    : f64 = 50.0;
const UI_BOX_BORD : f64 =  3.0;
const UI_MARGIN   : f64 =  5.0;
const UI_PADDING  : f64 =  3.0;
const UI_ELEM_N_H : f64 = 40.0;
const UI_ELEM_N_W : f64 = 30.0;

impl<'a, 'b> WeirdUIDrawer for CairoDrawer<'a, 'b> {
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
        println!("EXPOSED {:?}", expose);

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
        println!("EVENT!");

//        match ev.data {
//            EventType::MouseMove(_) => {
//                let pos = ev.pos();
//                println!("MOUSEMOVE: {}:{}", pos.x, pos.y);
//                self.post_redisplay();
//            },
//            _ => {},
//        }

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
