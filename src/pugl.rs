use pugl_sys::*;

use crate::ui::protocol::UIClientHandle;
use crate::ui::protocol::UIProviderHandle;
use crate::ui::constants::*;
use crate::ui::{UI, UIEvent};
use crate::ui;

const WINDOW_WIDTH:  usize = 700;
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

fn btn_num_to_uievent(num: usize) -> ui::MouseButton {
    match num {
        1 => ui::MouseButton::Left,
        2 => ui::MouseButton::Middle,
        3 => ui::MouseButton::Right,
        _ => ui::MouseButton::Left,
    }
}

fn key_to_uievent(key: pugl_sys::Key) -> ui::KeyEvent {
    match key.key {
        pugl_sys::KeyVal::Character(c) => {
            ui::KeyEvent {
                key: ui::Key::Character(c),
            }
        },
        pugl_sys::KeyVal::Special(special_key) => {
            let key =
                match special_key {
                    pugl_sys::SpecialKey::Backspace      => ui::Key::Backspace,
                    pugl_sys::SpecialKey::Escape         => ui::Key::Escape,
                    pugl_sys::SpecialKey::Delete         => ui::Key::Delete,
                    pugl_sys::SpecialKey::F1             => ui::Key::F1,
                    pugl_sys::SpecialKey::F2             => ui::Key::F2,
                    pugl_sys::SpecialKey::F3             => ui::Key::F3,
                    pugl_sys::SpecialKey::F4             => ui::Key::F4,
                    pugl_sys::SpecialKey::F5             => ui::Key::F5,
                    pugl_sys::SpecialKey::F6             => ui::Key::F6,
                    pugl_sys::SpecialKey::F7             => ui::Key::F7,
                    pugl_sys::SpecialKey::F8             => ui::Key::F8,
                    pugl_sys::SpecialKey::F9             => ui::Key::F9,
                    pugl_sys::SpecialKey::F10            => ui::Key::F10,
                    pugl_sys::SpecialKey::F11            => ui::Key::F11,
                    pugl_sys::SpecialKey::F12            => ui::Key::F12,
                    pugl_sys::SpecialKey::Left           => ui::Key::Left,
                    pugl_sys::SpecialKey::Up             => ui::Key::Up,
                    pugl_sys::SpecialKey::Right          => ui::Key::Right,
                    pugl_sys::SpecialKey::Down           => ui::Key::Down,
                    pugl_sys::SpecialKey::PageUp         => ui::Key::PageUp,
                    pugl_sys::SpecialKey::PageDown       => ui::Key::PageDown,
                    pugl_sys::SpecialKey::Home           => ui::Key::Home,
                    pugl_sys::SpecialKey::End            => ui::Key::End,
                    pugl_sys::SpecialKey::Insert         => ui::Key::Insert,
                    pugl_sys::SpecialKey::ShiftL         => ui::Key::ShiftL,
                    pugl_sys::SpecialKey::ShiftR         => ui::Key::ShiftR,
                    pugl_sys::SpecialKey::CtrlL          => ui::Key::CtrlL,
                    pugl_sys::SpecialKey::CtrlR          => ui::Key::CtrlR,
                    pugl_sys::SpecialKey::AltL           => ui::Key::AltL,
                    pugl_sys::SpecialKey::AltR           => ui::Key::AltR,
                    pugl_sys::SpecialKey::SuperL         => ui::Key::SuperL,
                    pugl_sys::SpecialKey::SuperR         => ui::Key::SuperR,
                    pugl_sys::SpecialKey::KeyMenu        => ui::Key::KeyMenu,
                    pugl_sys::SpecialKey::KeyCapsLock    => ui::Key::KeyCapsLock,
                    pugl_sys::SpecialKey::KeyScrollLock  => ui::Key::KeyScrollLock,
                    pugl_sys::SpecialKey::KeyNumLock     => ui::Key::KeyNumLock,
                    pugl_sys::SpecialKey::KeyPrintScreen => ui::Key::KeyPrintScreen,
                    pugl_sys::SpecialKey::KeyPause       => ui::Key::KeyPause,
                    pugl_sys::SpecialKey::None           => ui::Key::None,
                };
            ui::KeyEvent { key }
        },
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
            },
            EventType::MouseButtonRelease(btn) => {
                let ev_btn = btn_num_to_uievent(btn.num as usize);
                self.ui.handle_ui_event(UIEvent::MouseButtonReleased(ev_btn));
            },
            EventType::MouseButtonPress(btn) => {
                let ev_btn = btn_num_to_uievent(btn.num as usize);
                self.ui.handle_ui_event(UIEvent::MouseButtonPressed(ev_btn));
            },
            EventType::KeyPress(key) => {
                let ev_key = key_to_uievent(key);
                self.ui.handle_ui_event(UIEvent::KeyPressed(ev_key));
            },
            EventType::KeyRelease(key) => {
                let ev_key = key_to_uievent(key);
                self.ui.handle_ui_event(UIEvent::KeyReleased(ev_key));
            },
            _ => {
                println!("EVENT: {:?}", ev);
            },
        }

        if self.ui.needs_redraw() {
            self.post_redisplay();
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
