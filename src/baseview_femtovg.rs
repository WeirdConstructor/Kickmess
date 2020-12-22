use femtovg::{
    renderer::OpenGl,
    Canvas,
    Color,
};
use raw_gl_context::{GlContext, GlConfig, Profile};

use raw_window_handle::{
    unix::XlibHandle,
    HasRawWindowHandle,
    RawWindowHandle
};

#[macro_use]
use baseview::{
    Size, Event, MouseEvent, MouseButton, Parent, Window,
    WindowHandler, WindowOpenOptions, WindowScalePolicy,
    AppRunner,
};

use vst::plugin::{Info, Plugin};
use vst::editor::Editor;

use crate::ui::protocol::UIClientHandle;
use crate::ui::protocol::UIProviderHandle;
use crate::ui::constants::*;
use crate::ui::{UI, UIEvent};
use crate::ui;

const WINDOW_WIDTH:  usize = 512;
const WINDOW_HEIGHT: usize = 512;

pub struct TestWindowHandler {
//    ctx:        cairo::Context,
//    state:      PlugUIState,
//    display:    *mut x11::xlib::Display,
//    visual:     *mut x11::xlib::Visual,
//    drawable:   x11::xlib::Drawable,
//    gui_surf:   Option<cairo::Surface>,

    context:    GlContext,
    canvas:     Canvas<OpenGl>,
    ui:         UI,
}

impl WindowHandler for TestWindowHandler {

    fn on_event(&mut self, _: &mut Window, event: Event) {
        match event {
            Event::Mouse(MouseEvent::CursorMoved { position: p }) => {
                self.ui.handle_ui_event(UIEvent::MousePosition(p.x, p.y));
            },
            Event::Mouse(MouseEvent::ButtonPressed(btn)) => {
                let ev_btn =
                    match btn {
                        MouseButton::Left   => ui::MouseButton::Left,
                        MouseButton::Right  => ui::MouseButton::Right,
                        MouseButton::Middle => ui::MouseButton::Middle,
                        _                   => ui::MouseButton::Left,
                    };
                self.ui.handle_ui_event(UIEvent::MouseButtonPressed(ev_btn));
            },
            Event::Mouse(MouseEvent::ButtonReleased(btn)) => {
                let ev_btn =
                    match btn {
                        MouseButton::Left   => ui::MouseButton::Left,
                        MouseButton::Right  => ui::MouseButton::Right,
                        MouseButton::Middle => ui::MouseButton::Middle,
                        _                   => ui::MouseButton::Left,
                    };
                self.ui.handle_ui_event(UIEvent::MouseButtonReleased(ev_btn));
            },
            _ => {
                println!("UNHANDLED EVENT: {:?}", event);
            },
        }
    }

    fn on_frame(&mut self) {
        self.ui.handle_client_command();

        self.canvas.set_size(512, 512, 1.0);
        self.canvas.clear_rect(0, 0, 512, 512, Color::rgbf(0.3, 0.5, 0.32));

        self.canvas.flush();
        self.context.swap_buffers();

        if !self.ui.needs_redraw() {
            return;
        }

//        self.ui.draw(&gui_ctx);

    }
}

pub fn open_window(parent: Option<*mut ::std::ffi::c_void>, ui_hdl: UIProviderHandle) -> Option<AppRunner> {
    let options =
        if let Some(parent) = parent {
            let parent = raw_window_handle_from_parent(parent);
            WindowOpenOptions {
                title:  "BaseviewTest".to_string(),
                size:   Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale:  WindowScalePolicy::SystemScaleFactor,
                parent: Parent::WithParent(parent),
            }
        } else {
            WindowOpenOptions {
                title:  "BaseviewTest".to_string(),
                size:   Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale:  WindowScalePolicy::SystemScaleFactor,
                parent: Parent::None,
            }
        };

    Window::open(options, |win| {
        println!("XXX");
        let context =
            GlContext::create(
                win,
                GlConfig {
                    version:       (3, 2),
                    profile:       Profile::Core,
                    red_bits:      8,
                    blue_bits:     8,
                    green_bits:    8,
                    alpha_bits:    0,
                    depth_bits:    24,
                    stencil_bits:  8,
                    samples:       None,
                    srgb:          true,
                    double_buffer: true,
                    vsync:         false,
                }).unwrap();
        println!("XX2");
        context.make_current();
        println!("XX3");
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
        println!("XX4");

        let renderer =
            OpenGl::new(|symbol| context.get_proc_address(symbol) as *const _)
                .expect("Cannot create renderer");

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");

        let mut ui = UI::new(ui_hdl);

        ui.set_window_size(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);

        TestWindowHandler {
            ui,
            context,
            canvas,
//                    drawable: window,
//                    display: display as *mut x11::xlib::Display,
//                    visual: vis,
//                    gui_surf: None,
        }
//        unsafe {
//        }
    })
}

#[cfg(target_os = "macos")]
fn raw_window_handle_from_parent(
    parent: *mut ::std::ffi::c_void
) -> RawWindowHandle {
    use raw_window_handle::macos::MacOSHandle;
    use cocoa::base::id;
    use objc::{msg_send, sel, sel_impl};

    let ns_view = parent as id;

    let ns_window: id = unsafe {
        msg_send![ns_view, window]
    };

    RawWindowHandle::MacOS(MacOSHandle {
        ns_window: ns_window as *mut ::std::ffi::c_void,
        ns_view: ns_view as *mut ::std::ffi::c_void,
        ..MacOSHandle::empty()
    })
}


#[cfg(target_os = "windows")]
fn raw_window_handle_from_parent(
    parent: *mut ::std::ffi::c_void
) -> RawWindowHandle {
    use raw_window_handle::windows::WindowsHandle;

    RawWindowHandle::Windows(WindowsHandle {
        hwnd: parent,
        ..WindowsHandle::empty()
    })
}


#[cfg(target_os = "linux")]
fn raw_window_handle_from_parent(
    parent: *mut ::std::ffi::c_void
) -> RawWindowHandle {
    use raw_window_handle::unix::XcbHandle;

    RawWindowHandle::Xcb(XcbHandle {
        window: parent as u32,
        ..XcbHandle::empty()
    })
}
