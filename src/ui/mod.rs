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
    ui_handle: UIProviderHandle,
    painter:   Painter,
    layout:    Vec<UILayout>,
}

impl UI {
    pub fn new(ui_handle: UIProviderHandle) -> Self {
        Self {
            ui_handle,
            painter:    Painter::new(),
            layout:     vec![],
        }
    }

    pub fn handle_client_command(&mut self) {
        while let Ok(cmd) = self.ui_handle.rx.try_recv() {
            match cmd {
                UICmd::Define(layout) => {
                    self.layout = layout;
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
        // TODO:
        //  - calculate box sizes by 1/12th
        //  - distribute knobs thoughout the box available size.
        //  - make sure that it is possible to provide a minimal size
        //  - handle window resizing?
    }
}
