mod segmented_knob;
mod painting;
mod draw_cache;
mod constants;
pub mod protocol;

use crate::ui::painting::Painter;
use crate::ui::protocol::{UIMsg, UICmd, UIProviderHandle};

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
}

impl UI {
    pub fn new(ui_handle: UIProviderHandle) -> Self {
        Self {
            ui_handle,
            painter: Painter::new(),
        }
    }

    pub fn handle_client_command(&mut self) {
        // check ui_handle
    }

    pub fn handle_ui_event(&mut self, ev: UIEvent, cr: &cairo::Context) {
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
    }
}
