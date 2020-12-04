use crate::KickmessVSTParams;
use vst::editor::Editor;
use std::sync::Arc;
use std::rc::Rc;

use crate::ui::protocol::*;
use crate::ui::constants::*;
use crate::ui::{UI, UIEvent};
use crate::ui;

use pugl_sys::*;

const WINDOW_WIDTH:  usize = 500;
const WINDOW_HEIGHT: usize = 500;

struct KickmessUI {
    view:               PuglViewFFI,
    w:                  f64,
    h:                  f64,
    close_requested:    bool,
    ui:                 UI,
    cl_hdl:             UIClientHandle,
}

impl KickmessUI {
    fn new(view: PuglViewFFI) -> Self {
        let (cl_hdl, p_hdl) = UIClientHandle::create();

        let mut this = Self {
            view,
            w:               0.0,
            h:               0.0,
            close_requested: false,
            ui:              UI::new(p_hdl),
            cl_hdl,
        };

        this.ui.set_window_size(
            WINDOW_WIDTH  as f64,
            WINDOW_HEIGHT as f64);
        this.define_ui();

        this
    }

    fn define_ui(&mut self) {
        self.cl_hdl.tx.send(UICmd::Define(vec![
            UILayout::Container {
                label: String::from("Test"),
                xv: 1,
                yv: 1,
                wv: 10,
                hv: 10,
                elements: vec![
                    UIInput::Knob { label: String::from("SFreq."),     id: 1, xv: 0, yv: 0, },
                    UIInput::Knob { label: String::from("EFreq."),     id: 2, xv: 6, yv: 0, },
                    UIInput::Knob { label: String::from("Noise"),      id: 3, xv: 0, yv: 4, },
                    UIInput::Knob { label: String::from("Dist S."),    id: 4, xv: 6, yv: 4, },
                    UIInput::Knob { label: String::from("Dist E."),    id: 5, xv: 0, yv: 8, },
                    UIInput::Knob { label: String::from("Dist Gain"),  id: 6, xv: 6, yv: 8, },
                    UIInput::Knob { label: String::from("F Slope"),    id: 7, xv: 3, yv: 0, },
                    UIInput::Knob { label: String::from("Env Slope."), id: 8, xv: 3, yv: 4, },
                ],
            },
        ])).expect("mpsc ok");
    }
}

impl PuglViewTrait for KickmessUI {
    fn exposed(&mut self, expose: &ExposeArea, cr: &cairo::Context) {
        println!("EXPOSE");
        self.ui.draw(&cr);
//
//        cr.save();
//        cr.set_source_rgb(0.9, 0.9, 0.9);
//        cr.arc(100., 100., 60.0, 0.0, 4.0);
//        cr.stroke();
//        cr.restore();

//        let mut wd = PlugUIPainter::new(&mut self.state, cr);
//
//        let elems = [
//            Element::Knob(     0, 0),
//            Element::SmallKnob(1, 1),
//            Element::Toggle(   1, 2),
//        ];
//
//        let states = [
//            ElementState::Active(0.3),
//            ElementState::Disabled(0.5),
//            ElementState::Hover(0.5),
//        ];
//
//        wd.start_redraw();
//        wd.paint_element_hbox("Frequency", 0, 0, &elems, &states);
//        wd.done_redraw();
    }

    fn event(&mut self, ev: Event) -> Status {
//        println!("EVENT!");

        match ev.data {
            EventType::MouseMove(_) => {
                let pos = ev.pos();
                //d// println!("MOUSEMOVE: {}:{}", pos.x, pos.y);
                self.ui.handle_ui_event(UIEvent::MousePosition(pos.x, pos.y));
                self.post_redisplay();
            },
            EventType::MouseButtonRelease(btn) => {
                let ev_btn =
                    match btn.num {
                        1 => ui::MouseButton::Left,
                        2 => ui::MouseButton::Middle,
                        3 => ui::MouseButton::Right,
                        _ => ui::MouseButton::Left,
                    };
                self.ui.handle_ui_event(UIEvent::MouseButtonReleased(ev_btn));
            },
            EventType::MouseButtonPress(btn) => {
                let ev_btn =
                    match btn.num {
                        1 => ui::MouseButton::Left,
                        3 => ui::MouseButton::Middle,
                        2 => ui::MouseButton::Right,
                        _ => ui::MouseButton::Left,
                    };
                self.ui.handle_ui_event(UIEvent::MouseButtonPressed(ev_btn));
            },
            _ => {},
        }

        Status::Success
    }

    fn resize(&mut self, size: Size) {
        println!("RESIZE {:?}", size);
        self.ui.set_window_size(
            size.w as f64,
            size.h as f64);
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
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
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
            size: Size { w: WINDOW_WIDTH as f64, h: WINDOW_HEIGHT as f64 },
        });
        println!("title: {:?}", ui.set_window_title("Kickmess"));
        ui.make_resizable();
        println!("set_default_size: {:?}",
            ui.set_default_size(
                WINDOW_WIDTH as i32,
                WINDOW_HEIGHT as i32));
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
            hdl.update(0.0);
//            hdl.draw();
    //        println!("IDLE!?!");

            while let Ok(msg) = hdl.cl_hdl.rx.try_recv() {
                println!("MSG FROM UI: {:?}", msg);
            }
            hdl.ui.handle_client_command();

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
