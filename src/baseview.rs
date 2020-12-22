#[macro_use]
use baseview::{
    Size, Event, MouseEvent, MouseButton, Parent, Window,
    WindowHandler, WindowOpenOptions, WindowScalePolicy,
    AppRunner,
};

use raw_window_handle::{
    unix::XlibHandle,
    HasRawWindowHandle,
    RawWindowHandle
};

use vst::plugin::{Info, Plugin};
use vst::editor::Editor;

use crate::ui::protocol::UIClientHandle;
use crate::ui::protocol::UIProviderHandle;
use crate::ui::constants::*;
use crate::ui::{UI, UIEvent};
use crate::ui;

const WINDOW_WIDTH:  usize = 500;
const WINDOW_HEIGHT: usize = 500;

pub struct TestWindowHandler {
//    ctx:        cairo::Context,
//    state:      PlugUIState,
    display:    *mut x11::xlib::Display,
    visual:     *mut x11::xlib::Visual,
    drawable:   x11::xlib::Drawable,
    gui_surf:   Option<cairo::Surface>,
    ui:         UI,
}

fn new_screen_buffer(cr: &cairo::Context) -> (cairo::Surface, cairo::Context) {
    let ext = cr.clip_extents();
    let surf =
        cr.get_target()
          .create_similar_image(
              cairo::Format::ARgb32,
              (ext.0 - ext.2).abs() as i32,
              (ext.1 - ext.3).abs() as i32)
          .expect("Createable new img surface");
    let ctx = cairo::Context::new(&surf);
    (surf, ctx)
}

impl WindowHandler for TestWindowHandler {
    type Message = ();

    fn on_message(&mut self, _: &mut Window, message: Self::Message) {
        println!("MESG: {:?}", message);
    }

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

        if !self.ui.needs_redraw() {
            return;
        }

        let surf =
            unsafe {
                cairo_sys::cairo_xlib_surface_create(
                    self.display,
                    self.drawable,
                    self.visual,
                    WINDOW_WIDTH  as i32,
                    WINDOW_HEIGHT as i32
                )
            };

        let surf =
            unsafe {
                cairo::Surface::from_raw_full(surf)
                    .expect("surface creation from xlib surface ok") };
        let ctx = cairo::Context::new(&surf);
        let ext = ctx.clip_extents();

        let front_surf =
            ctx.get_target()
              .create_similar(
                  cairo::Content::ColorAlpha,
                  (ext.0 - ext.2).abs() as i32,
                  (ext.1 - ext.3).abs() as i32)
              .expect("Createable new img surface");
        let mut front_ctx = cairo::Context::new(&front_surf);

        let flush =
            if self.ui.needs_redraw() {
                let mut gui_ctx = front_ctx;

                let ext = ctx.clip_extents();
                gui_ctx.set_source_rgb(
                    UI_GUI_CLEAR_CLR.0,
                    UI_GUI_CLEAR_CLR.1,
                    UI_GUI_CLEAR_CLR.2);
                gui_ctx.rectangle(ext.0, ext.1, ext.2 - ext.0, ext.3 - ext.1);
                gui_ctx.fill();
                self.ui.draw(&gui_ctx);
                true
            } else {
                false
            };

        ctx.set_source_surface(&front_surf, 0.0, 0.0);
        ctx.paint();
        surf.flush();

        if flush {
            println!("XFlush!");
            unsafe { x11::xlib::XFlush(self.display); }
        }
    }
}

pub fn open_window(parent: Option<*mut ::std::ffi::c_void>, ui_hdl: UIProviderHandle) -> (WindowHandle<TestWindowHandler>, Option<AppRunner>) {
    let options =
        if let Some(parent) = parent {
            let parent = raw_window_handle_from_parent(parent);
            WindowOpenOptions {
                title:  "BaseviewTest".to_string(),
                size:   Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale:  WindowScalePolicy::ScaleFactor(1.0),
                parent: Parent::WithParent(parent),
            }
        } else {
            WindowOpenOptions {
                title:  "BaseviewTest".to_string(),
                size:   Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale:  WindowScalePolicy::ScaleFactor(1.0),
                parent: Parent::None,
            }
        };

    Window::open(options, |win| {
        unsafe {
            if let RawWindowHandle::Xlib(XlibHandle {
                    window, display,
                    ..
                }) = win.raw_window_handle()
            {

                let vis =
                    x11::xlib::XDefaultVisual(
                        display as *mut x11::xlib::Display,
                        x11::xlib::XDefaultScreen(
                            display as *mut x11::xlib::Display));

                let mut ui = UI::new(ui_hdl);

                ui.set_window_size(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);

                TestWindowHandler {
                    ui,
                    drawable: window,
                    display: display as *mut x11::xlib::Display,
                    visual: vis,
                    gui_surf: None,
                }
            }
            else
            {
                panic!("Can only handle XlibHandle!");
            }
        }
    })
}

#[derive(Default)]
struct TestPluginEditor {
    handle: Option<(WindowHandle<TestWindowHandler>, Option<AppRunner>)>,
    cl_hdl: Option<UIClientHandle>,
}


impl Editor for TestPluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        let (cl_hdl, p_hdl) = UIClientHandle::create();
        self.cl_hdl = Some(cl_hdl);
        self.handle = Some(open_window(Some(parent), p_hdl));
        true
    }

    fn is_open(&mut self) -> bool {
        self.handle.is_some()
    }

    fn close(&mut self) {
        self.handle = None;
    }
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
