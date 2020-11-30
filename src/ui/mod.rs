mod segmented_knob;
mod painting;
mod draw_cache;
mod constants;
pub mod protocol;

use crate::ui::painting::Painter;
use crate::ui::protocol::{UIMsg, UICmd, UIProviderHandle, UILayout};

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub enum UIEvent {
    MousePosition(f64, f64),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
}

pub struct UI {
    ui_handle:      UIProviderHandle,
    painter:        Painter,
    layout:         Vec<UILayout>,
    window_size:    (f64, f64),
}

impl UI {
    pub fn new(ui_handle: UIProviderHandle) -> Self {
        Self {
            ui_handle,
            painter:        Painter::new(),
            layout:         vec![],
            window_size:    (0.0, 0.0),
        }
    }

    pub fn set_window_size(&mut self, w: f64, h: f64) {
        self.window_size = (w, h);
    }

    pub fn handle_client_command(&mut self) {
        while let Ok(cmd) = self.ui_handle.rx.try_recv() {
            match cmd {
                UICmd::Define(layout) => {
                    self.layout = layout;
                    println!("CLIENT EVENT: LAYOUT!");
                },
                UICmd::SetValues(_) => {
                },
            }
        }
        // check ui_handle
    }

    pub fn handle_ui_event(&mut self, ev: UIEvent) {
        match ev {
            UIEvent::MousePosition(x, y) => {
                for zone in self.painter.zones.iter() {
                    if zone.is_inside(x, y) {
//                        self.hover_zone = Some(zone.id);
                        println!("handle_mouse: {},{} => Hoverzone={}",
                                 x, y, zone.id);
                        break;
                    }
                }
            },
            _ => {},
        }
    }

    pub fn draw(&mut self, cr: &cairo::Context) {
        let (ww, wh) = self.window_size;

        for layout in self.layout.iter() {
            match layout {
                UILayout::Container { label, xv, yv, wv, hv, elements } => {
                    let x = (((*xv as f64) * ww) / 12.0).floor();
                    let y = (((*yv as f64) * wh) / 12.0).floor();
                    let w = (((*wv as f64) * ww) / 12.0).ceil();
                    let h = (((*hv as f64) * wh) / 12.0).ceil();

                    cr.set_source_rgb(0.29, 0.29, 0.29);
                    cr.rectangle(x, y, w, h);
                    cr.fill();
                    println!("DRAW CONTAINER {},{},{},{}", x, y, w, h);
                },
            }
        }
        // TODO:
        //  - calculate box sizes by 1/12th
        //  - distribute knobs thoughout the box available size.
        //  - make sure that it is possible to provide a minimal size
        //  - handle window resizing?
    }
}
