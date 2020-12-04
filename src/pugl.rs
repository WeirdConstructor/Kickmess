use pugl_sys::*;

use crate::ui::protocol::UIClientHandle;
use crate::ui::protocol::UIProviderHandle;
use crate::ui::constants::*;
use crate::ui::{UI, UIEvent};
use crate::ui;

const WINDOW_WIDTH:  usize = 500;
const WINDOW_HEIGHT: usize = 500;

pub struct PuglUI {
    view:               PuglViewFFI,
    ui:                 UI,
    cl_hdl:             Option<UIClientHandle>,
}

impl PuglUI {
    pub fn new(view: PuglViewFFI, ui_hdl: UIProviderHandle) -> Self {
        Self {
            view,
            ui:              UI::new(ui_hdl),
            cl_hdl:          None,
        }
    }

    pub fn new_with_internal_handles(view: PuglViewFFI) -> Self {
        let (cl_hdl, p_hdl) = ui::protocol::UIClientHandle::create();

        Self {
            view,
            ui:              UI::new(p_hdl),
            cl_hdl:          Some(cl_hdl),
        }
    }

    pub fn cl_hdl(&self) -> Option<&UIClientHandle> { self.cl_hdl.as_ref() }

    pub fn update_ui(&mut self) {
        self.ui.handle_client_command();
    }
}

impl PuglViewTrait for PuglUI {
    fn exposed(&mut self, expose: &ExposeArea, cr: &cairo::Context) {
        //d// println!("EXPOSE!");
        self.ui.draw(&cr);
    }

    fn event(&mut self, ev: Event) -> Status {
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
            _ => {
                println!("EVENT: {:?}", ev);
            },
        }

        Status::Success
    }

    fn resize(&mut self, size: Size) {
        self.ui.set_window_size(
            size.w as f64,
            size.h as f64);
    }

    fn close_request(&mut self) {
        self.ui.handle_ui_event(UIEvent::WindowClose);
    }

    fn view(&self) -> PuglViewFFI {
        self.view
    }
}

pub fn open_window(parent: Option<*mut std::ffi::c_void>, ui_hdl: Option<UIProviderHandle>) -> Box<PuglView<PuglUI>> {
    let mut view =
        if let Some(ui_hdl) = ui_hdl {
            PuglView::<PuglUI>::new(
                if let Some(parent) = parent { parent }
                else                         { std::ptr::null_mut() },
                move |pv| PuglUI::new(pv, ui_hdl))
        } else {
            PuglView::<PuglUI>::new(
                if let Some(parent) = parent { parent }
                else                         { std::ptr::null_mut() },
                move |pv| PuglUI::new_with_internal_handles(pv))
        };

    let ui = view.handle();
    ui.set_frame(Rect {
        pos: Coord { x: 0., y: 0. },
        size: Size { w: WINDOW_WIDTH as f64, h: WINDOW_HEIGHT as f64 },
    });
    ui.set_window_title("Kickmess");
//    ui.make_resizable();
    ui.set_default_size(
        WINDOW_WIDTH as i32,
        WINDOW_HEIGHT as i32);
    ui.show_window();

    view
}
