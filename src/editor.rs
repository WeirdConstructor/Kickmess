use crate::KickmessVSTParams;
use vst::editor::Editor;
use std::sync::Arc;

use pugl_sys::*;


struct KickmessUI {
    view:               PuglViewFFI,
    w:                  f64,
    h:                  f64,
    close_requested:    bool,
}

impl KickmessUI {
    fn new(view: PuglViewFFI) -> Self {
        Self {
            view,
            w:               0.0,
            h:               0.0,
            close_requested: false,
        }
    }
}

impl PuglViewTrait for KickmessUI {
    fn exposed(&mut self, expose: &ExposeArea, cr: &cairo::Context) {
        println!("EXPOSED {:?}", expose);

        cr.set_source_rgb(0.2, 0.2, 0.2);
        cr.rectangle(0., 0., self.w, self.h);
        cr.fill();

        cr.set_source_rgb(0.9, 0.9, 0.9);
        cr.arc(self.w / 4.0, self.h / 4.0, 40.0, 0.0, 6.0);
        cr.fill();
    }

    fn event(&mut self, ev: Event) -> Status {
        println!("EVENT!");

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
        self.w = size.w;
        self.h = size.h;
        self.post_redisplay();
    }

    fn close_request(&mut self) {
        println!("CLOSE REQ");
        self.close_requested = true;
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
        (400, 400)
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
        println!("OPEN null={}", parent.is_null());
        let mut view =
            PuglView::<KickmessUI>::new(
                parent,
                |pv| KickmessUI::new(pv));

        let ui = view.handle();
        println!("title: {:?}", ui.set_window_title("Kickmess"));
        ui.make_resizable();
        println!("set_default_size: {:?}", ui.set_default_size(900, 400));
        println!("show_window: {:?}", ui.realize());
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
            println!("update: {:?}", hdl.update(0.0));
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
