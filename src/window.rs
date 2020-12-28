// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use femtovg::{
    renderer::OpenGl,
    Canvas,
    FontId,
    ImageId,
    Color,
};
use raw_gl_context::{GlContext, GlConfig, Profile};

use raw_window_handle::{
    HasRawWindowHandle,
    RawWindowHandle
};

#[macro_use]
use baseview::{
    Size, Event, WindowEvent, WindowInfo, MouseEvent, MouseButton, Parent, Window,
    WindowHandler, WindowOpenOptions, WindowScalePolicy, AppRunner,
};

use vst::plugin::{Info, Plugin};
use vst::editor::Editor;

use crate::ui::protocol::UIClientHandle;
use crate::ui::protocol::UIProviderHandle;
use crate::ui::constants::*;
use crate::ui::painting::Painter;
use crate::ui::{UI, UIEvent};
use crate::ui;

struct FrameTimeMeasurement {
    buf: [u128; 60],
    idx: usize,
    cur: Option<std::time::Instant>,
    lbl: String,
}

impl FrameTimeMeasurement {
    fn new(lbl: &str) -> Self {
        Self {
            buf: [0; 60],
            idx: 0,
            cur: None,
            lbl: lbl.to_string(),
        }
    }

    fn start_measure(&mut self) {
        self.cur = Some(std::time::Instant::now());
    }

    fn end_measure(&mut self) {
        if let Some(cur) = self.cur.take() {
            let dur_microseconds = cur.elapsed().as_micros();
            if (self.idx + 1) >= self.buf.len() {
                let mut min = 99999999;
                let mut max = 0;
                let mut avg = 0;
                for b in self.buf.iter() {
                    if *b < min { min = *b; }
                    if *b > max { max = *b; }
                    avg += *b;
                }
                avg /= self.buf.len() as u128;
                println!("Frame time [{:10}]: min={:5.3}, max={:5.3}, avg={:5.3}", self.lbl, min as f64 / 1000.0, max as f64 / 1000.0, avg as f64 / 1000.0);
                self.idx = 0;
            } else {
                self.idx += 1;
            }
            self.buf[self.idx] = dur_microseconds;
        }
    }
}

pub struct GUIWindowHandler {
    context:    GlContext,
    canvas:     Canvas<OpenGl>,
    font:       FontId,
    font_mono:  FontId,
    img_buf:    ImageId,
    ftm:        FrameTimeMeasurement,
    ftm_redraw: FrameTimeMeasurement,
    ui:         UI,
    scale:      f32,
    size:       (f64, f64),
}

struct MyPainter<'a> {
    canvas:     &'a mut Canvas<OpenGl>,
    font:       FontId,
    font_mono:  FontId,
}

fn color_paint(color: (f64, f64, f64)) -> femtovg::Paint {
    femtovg::Paint::color(
        Color::rgbf(
            color.0 as f32,
            color.1 as f32,
            color.2 as f32))
}

impl<'a> MyPainter<'a> {
    fn label_with_font(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str, font: FontId) {
        let mut paint = color_paint(color);
        paint.set_font(&[font]);
        paint.set_font_size(1.0 + size as f32);
        paint.set_text_baseline(femtovg::Baseline::Middle);
        let x = x.round();
        match align {
            -1 => {
                paint.set_text_align(femtovg::Align::Left);
                self.canvas.fill_text(x as f32, (y + h / 2.0).round() as f32, text, paint);
            },
            0  => {
                paint.set_text_align(femtovg::Align::Center);
                self.canvas.fill_text((x + (w / 2.0)) as f32, (y + h / 2.0).round() as f32, text, paint);
            },
            _  => {
                paint.set_text_align(femtovg::Align::Right);
                self.canvas.fill_text((x + w) as f32, (y + h / 2.0).round() as f32, text, paint);
            },
        }
    }
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
        self.label_with_font(size, align, color, x, y, w, h, text, self.font);
    }

    fn label_mono(&mut self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str) {
        self.label_with_font(size, align, color, x, y, w, h, text, self.font_mono);
    }
}

impl WindowHandler for GUIWindowHandler {

    fn on_event(&mut self, _: &mut Window, event: Event) {
        match event {
            Event::Mouse(MouseEvent::CursorMoved { position: p }) => {
                self.ui.handle_ui_event(
                    UIEvent::MousePosition(
                        p.x / self.scale as f64,
                        p.y / self.scale as f64));
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
            Event::Keyboard(ev) => {
                use keyboard_types::KeyState;
                match ev.state {
                    KeyState::Up => {
                        self.ui.handle_ui_event(UIEvent::KeyReleased(ev));
                    },
                    KeyState::Down => {
                        self.ui.handle_ui_event(UIEvent::KeyPressed(ev));
                    },
                }
            },
            Event::Window(WindowEvent::Resized(info)) => {
                println!("RESIZE EVENT: {:?}", event);
                let size = info.logical_size();

                self.canvas.set_size(size.width as u32, size.height as u32, 1.0);
                let (w, h) = (self.canvas.width(), self.canvas.height());
                self.canvas.delete_image(self.img_buf);
                self.img_buf =
                    self.canvas.create_image_empty(
                        w as usize, h as usize,
                        femtovg::PixelFormat::Rgb8,
                        femtovg::ImageFlags::FLIP_Y).expect("making image buffer");

                let ri = (w as f32) / (h as f32);
                let rs = (self.size.0 as f32) / (self.size.1 as f32);

                if rs > ri {
                    self.scale = (w as f32) / (self.size.0 as f32);
                } else {
                    self.scale = (h as f32) / (self.size.1 as f32);
                }

                self.ui.queue_redraw();
            },
            _ => {
                println!("UNHANDLED EVENT: {:?}", event);
            },
        }
    }

    fn on_frame(&mut self) {
        self.ui.handle_client_command();

        let redraw = self.ui.needs_redraw();

        if redraw {
            self.ftm.start_measure();
        }

        if redraw {
            self.ftm_redraw.start_measure();
            self.canvas.set_render_target(femtovg::RenderTarget::Image(self.img_buf));
            self.canvas.save();
            self.canvas.scale(self.scale, self.scale);
            self.canvas.clear_rect(
                0, 0,
                self.canvas.width() as u32,
                self.canvas.height() as u32,
                Color::rgbf(
                    UI_GUI_BG_CLR.0 as f32,
                    UI_GUI_BG_CLR.1 as f32,
                    UI_GUI_BG_CLR.2 as f32));
            self.ui.draw(&mut MyPainter {
                canvas:     &mut self.canvas,
                font:       self.font,
                font_mono:  self.font_mono,
            });
            self.canvas.restore();
            self.ftm_redraw.end_measure();
        }


        let img_paint =
            femtovg::Paint::image(
                self.img_buf, 0.0, 0.0,
                self.canvas.width(),
                self.canvas.height(),
                0.0, 1.0);
        let mut path = femtovg::Path::new();
        path.rect(0.0, 0.0, self.canvas.width(), self.canvas.height());

        self.canvas.set_render_target(femtovg::RenderTarget::Screen);
        self.canvas.fill_path(&mut path, img_paint);

        self.canvas.flush();
        self.context.swap_buffers();

        if redraw {
            self.ftm.end_measure();
        }
    }
}

pub fn open_window(title: &str, window_width: i32, window_height: i32, parent: Option<*mut ::std::ffi::c_void>, ui_hdl: UIProviderHandle) -> Option<AppRunner> {
    let options =
        if let Some(parent) = parent {
            let parent = raw_window_handle_from_parent(parent);
            WindowOpenOptions {
                title:  title.to_string(),
                size:   Size::new(window_width as f64, window_height as f64),
                scale:  WindowScalePolicy::SystemScaleFactor,
                parent: Parent::WithParent(parent),
            }
        } else {
            WindowOpenOptions {
                title:  title.to_string(),
                size:   Size::new(window_width as f64, window_height as f64),
                scale:  WindowScalePolicy::SystemScaleFactor,
                parent: Parent::None,
            }
        };

    Window::open(options, move |win| {
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
                    vsync:         true,
                }).unwrap();
        context.make_current();
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        let renderer =
            OpenGl::new(|symbol| context.get_proc_address(symbol) as *const _)
                .expect("Cannot create renderer");

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
        canvas.set_size(window_width as u32, window_height as u32, 1.0);
        let font      = canvas.add_font_mem(std::include_bytes!("font.ttf")).expect("can load font");
        let font_mono = canvas.add_font_mem(std::include_bytes!("font_mono.ttf")).expect("can load font");
        let (w, h) = (canvas.width(), canvas.height());
        let img_buf =
            canvas.create_image_empty(
                w as usize, h as usize,
                femtovg::PixelFormat::Rgb8,
                femtovg::ImageFlags::FLIP_Y).expect("making image buffer");

        let mut ui = UI::new(ui_hdl);

        ui.set_window_size(window_width as f64, window_height as f64);

        GUIWindowHandler {
            ui,
            size: (window_width as f64, window_height as f64),
            context,
            canvas,
            font,
            font_mono,
            img_buf,
            ftm:        FrameTimeMeasurement::new("img"),
            ftm_redraw: FrameTimeMeasurement::new("redraw"),
            scale:      1.0,
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
