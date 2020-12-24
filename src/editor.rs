use crate::KickmessVSTParams;
use vst::editor::Editor;
use std::sync::Arc;
use std::rc::Rc;

use crate::ui::protocol::*;
use crate::ui::constants::*;
use crate::ui::{UI, UIEvent};
use crate::ui;

const WINDOW_WIDTH:  usize = 500;
const WINDOW_HEIGHT: usize = 500;

pub(crate) struct KickmessEditor {
//    view:      Option<Box<PuglView<PuglUI>>>,
    params:    Arc<KickmessVSTParams>,
}

impl KickmessEditor {
    pub(crate) fn new(params: Arc<KickmessVSTParams>) -> Self {
        Self {
//            view: None,
            params,
        }
    }
}

impl Editor for KickmessEditor {
    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn open(&mut self, parent: *mut std::ffi::c_void) -> bool {
        println!("OPEN null={} == {:?}", parent.is_null(), parent);
//        self.view = Some(open_window(Some(parent), None));

        println!("OPENED");

        true
    }

    fn is_open(&mut self) -> bool {
//        self.view.is_some()
        false
    }

    fn idle(&mut self) {
//        let mut close = false;
//
//        if let Some(view) = self.view.as_mut() {
//            let hdl = view.as_mut().handle();
//
//            hdl.update(0.0);
//
//            while let Ok(msg) = hdl.cl_hdl().unwrap().rx.try_recv() {
//                match msg {
//                    UIMsg::WindowClosed => { close = true; },
//                    _ => {
//                        println!("MSG FROM UI: {:?}", msg);
//                    }
//                }
//            }
//
//            hdl.update_ui();
//        }
//
//        if close {
//            self.view = None;
//        }
    }

    fn close(&mut self) {
//        self.view.as_mut().unwrap().as_mut().handle().close_request()
    }
}
