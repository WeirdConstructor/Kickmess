use femtovg::{
    renderer::OpenGl,
    Canvas,
    FontId,
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
use crate::ui::painting::Painter;
use crate::ui::{UI, UIEvent};
use crate::ui;

const WINDOW_WIDTH:  usize = 800;
const WINDOW_HEIGHT: usize = 512;

pub struct TestWindowHandler {
    context:    GlContext,
    canvas:     Canvas<OpenGl>,
    font:       FontId,
    ui:         UI,
}

struct MyPainter<'a> {
    canvas: &'a mut Canvas<OpenGl>,
    font:   FontId,
}

fn color_paint(color: (f64, f64, f64)) -> femtovg::Paint {
    femtovg::Paint::color(
        Color::rgbf(
            color.0 as f32,
            color.1 as f32,
            color.2 as f32))
}

impl<'a> Painter for MyPainter<'a> {
    fn path_fill(&mut self, color: (f64, f64, f64), segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>, closed: bool) {
        let mut p = femtovg::Path::new();
        let mut paint = color_paint(color);

        let mut first = true;
        for s in segments {
            if first {
                p.move_to(s.0 as f32, s.1 as f32);
                first = false;
            } else {
                p.line_to(s.0 as f32, s.1 as f32);
            }
        }

        if closed { p.close(); }

        self.canvas.fill_path(&mut p, paint);
    }

    fn path_stroke(&mut self, width: f64, color: (f64, f64, f64), segments: &mut dyn std::iter::Iterator<Item = (f64, f64)>, closed: bool) {
        let mut p = femtovg::Path::new();
        let mut paint = color_paint(color);
        paint.set_line_width(width as f32);

        let mut first = true;
        for s in segments {
            if first {
                p.move_to(s.0 as f32, s.1 as f32);
                first = false;
            } else {
                p.line_to(s.0 as f32, s.1 as f32);
            }
        }

        if closed { p.close(); }

        self.canvas.stroke_path(&mut p, paint);
    }

    fn arc_stroke(&mut self, width: f64, radius: f64, from_rad: f64, to_rad: f64, x: f64, y: f64) {
    }

    fn rect_fill(&mut self, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64) {
        let mut p = femtovg::Path::new();
        p.rect(x as f32, y as f32, w as f32, h as f32);
        self.canvas.fill_path(&mut p, color_paint(color));
    }

    fn rect_stroke(&mut self, width: f64, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64) {
        let mut p = femtovg::Path::new();
        p.rect(x as f32, y as f32, w as f32, h as f32);
        let mut paint = color_paint(color);
        paint.set_line_width(width as f32);
        self.canvas.stroke_path(&mut p, paint);
    }

    fn label(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str) {
        let mut paint = color_paint(color);
        paint.set_font_size(1.0 + size as f32);
        paint.set_text_baseline(femtovg::Baseline::Middle);
        let x = x.round();
        match align {
            -1 => {
                paint.set_text_align(femtovg::Align::Left);
                self.canvas.fill_text(x as f32, (y + h / 2.0) as f32, text, paint);
            },
            0  => {
                paint.set_text_align(femtovg::Align::Center);
                self.canvas.fill_text((x + (w / 2.0)) as f32, (y + h / 2.0) as f32, text, paint);
            },
            _  => {
                paint.set_text_align(femtovg::Align::Right);
                self.canvas.fill_text((x + w) as f32, (y + h / 2.0) as f32, text, paint);
            },
        }
        //d// self.rect_stroke(0.5, (1.0, 0.0, 1.0), x, y, w, h);
//        println!("DRAW LABEL {}", text);
    }
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

        self.canvas.set_size(800, 512, 1.0);
        self.canvas.clear_rect(0, 0, 800, 512, Color::rgbf(0.3, 0.5, 0.32));

        let mut p = femtovg::Path::new();
        p.move_to(10.0, 10.0);
        p.line_to(100.0, 200.0);
        let mut paint = femtovg::Paint::color(Color::rgbf(1.0, 0.0, 1.0));
        paint.set_line_width(2.0);
        self.canvas.stroke_path(&mut p, paint);

        self.ui.draw(&mut MyPainter { canvas: &mut self.canvas, font: self.font });

        self.canvas.flush();
        self.context.swap_buffers();
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
        let font = canvas.add_font("DejaVuSerif.ttf").expect("can load font");

        let mut ui = UI::new(ui_hdl);

        ui.set_window_size(WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64);

        TestWindowHandler {
            ui,
            context,
            canvas,
            font,
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
