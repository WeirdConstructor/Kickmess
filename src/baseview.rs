#[macro_use]
extern crate vst;

use baseview::{
    Size, Event, Parent, Window, WindowHandle, WindowHandler,
    WindowOpenOptions, WindowScalePolicy
}
use raw_window_handle::{
    unix::XlibHandle,
    HasRawWindowHandle,
    RawWindowHandle
};;

use raw_window_handle::RawWindowHandle;
use vst::plugin::{Info, Plugin};
use vst::editor::Editor;


const WINDOW_WIDTH: usize = 500;
const WINDOW_HEIGHT: usize = 500;


#[derive(Default)]
struct TestWindowHandler;


impl WindowHandler for TestWindowHandler {
    type Message = ();

    fn on_message(&mut self, _: &mut Window, message: Self::Message) {
        ::log::info!("TestWindowHandler received message: {:?}", message)
    }

    fn on_event(&mut self, _: &mut Window, event: Event) {
        ::log::info!("TestWindowHandler received event: {:?}", event)
    }

    fn on_frame(&mut self) {
        
    }
}

fn open_window(parent: Option<*mut ::std::ffi::c_void>) -> WindowHandle {
    let parent = raw_window_handle_from_parent(parent);

    let options =
        if let Some(parent) = parent {
            WindowOpenOptions {
                title: "BaseviewTest".to_string(),
                size: Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale: WindowScalePolicy::ScaleFactor(1.0),
                parent: Parent::WithParent(parent),
            }
        } else {
            WindowOpenOptions {
                title: "BaseviewTest".to_string(),
                size: Size::new(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64),
                scale: WindowScalePolicy::ScaleFactor(1.0),
                parent: Parent::None,
            }
        };

    Window::open(options, |win|{
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

                let surf =
                    cairo_sys::cairo_xlib_surface_create(
                        display as *mut x11::xlib::Display,
                        window,
                        vis,
                        WINDOW_WIDTH  as i32,
                        WINDOW_HEIGHT as i32
                    );

                let surf =
                    cairo::Surface::from_raw_full(surf)
                        .expect("surface creation from xlib surface ok");
                let ctx = cairo::Context::new(&surf);

                TestWindowHandler {
                    once:false,
                    surf,
                    ctx
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
    handle: Option<WindowHandle>,
}


impl Editor for TestPluginEditor {
    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut ::std::ffi::c_void) -> bool {
        self.handle = open_window_parent(parent);
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
