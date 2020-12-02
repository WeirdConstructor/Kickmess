mod segmented_knob;
mod painting;
mod draw_cache;
pub mod constants;
pub mod protocol;

use std::rc::Rc;
use std::cell::RefCell;

use crate::ui::painting::{Painter, ActiveZone};
use crate::ui::draw_cache::DrawCache;
use crate::ui::protocol::{UIMsg, UICmd, UIProviderHandle, UILayout, UIInput};
use crate::ui::constants::*;

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub enum UIEvent {
    MousePosition(f64, f64),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
}

pub struct UI {
    ui_handle:      UIProviderHandle,

//    painter:        Painter,

    layout:         Rc<RefCell<Vec<UILayout>>>,

    element_values: Vec<f32>,
    window_size:    (f64, f64),

    zones:          Vec<ActiveZone>,
    cache:          DrawCache,

    hover_zone:     Option<ActiveZone>,
    last_mouse_pos: (f64, f64),
}

impl UI {
    pub fn new(ui_handle: UIProviderHandle) -> Self {
        Self {
            ui_handle,
//            painter:        Painter::new(),
            layout:             Rc::new(RefCell::new(vec![])),
            window_size:        (0.0, 0.0),
            zones:              vec![],
            cache:              DrawCache::new(),
            element_values:     vec![],
            hover_zone:         None,
            last_mouse_pos:     (0.0, 0.0),
        }
    }

    pub fn set_window_size(&mut self, w: f64, h: f64) {
        self.window_size = (w, h);
    }

    pub fn handle_client_command(&mut self) {
        while let Ok(cmd) = self.ui_handle.rx.try_recv() {
            match cmd {
                UICmd::Define(layout) => {
                    self.layout = Rc::new(RefCell::new(layout));
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
                self.last_mouse_pos = (x, y);
                self.hover_zone     = None;

                for zone in self.zones.iter() {
                    if zone.is_inside(x, y) {
                        self.hover_zone = Some(*zone);
                        println!("handle_mouse: {},{} => Hoverzone={}",
                                 x, y, zone.id);
                        break;
                    }
                }
            },
            UIEvent::MouseButtonPressed(btn) => {
                println!("BUTTON PRESS: {:?} @{:?}", btn, self.last_mouse_pos);
            },
            UIEvent::MouseButtonReleased(btn) => {
                println!("BUTTON RELEASE: {:?} @{:?}", btn, self.last_mouse_pos);
            },
            _ => {},
        }
    }

//    fn get_element_value_formatted(&mut self, id: usize) -> &str {
//        if id >= self.element_values_str.len() {
//            self.element_values_str.resize(id * 2, String::new());
//        }
//
//        let v = self.get_element_value(id);
//        self.element_values_str[id] = format!("{}", v);
//
//        &self.element_values_str[id]
//    }

    fn get_element_value(&mut self, id: usize) -> f32 {
        if id >= self.element_values.len() {
            self.element_values.resize(id * 2, 0.7);
        }

        self.element_values[id]
    }

    fn add_active_zone(&mut self, id: usize, mut az: ActiveZone) {
        az.id = id;
        self.zones.push(az);
    }

    pub fn draw(&mut self, cr: &cairo::Context) {
        let (ww, wh) = self.window_size;

        self.zones.clear();

        let layout = self.layout.clone();

        for layout in layout.borrow_mut().iter() {
            match layout {
                UILayout::Container { label, xv, yv, wv, hv, elements } => {
                    let x = (((*xv as f64) * ww) / 12.0).floor();
                    let y = (((*yv as f64) * wh) / 12.0).floor();
                    let w = (((*wv as f64) * ww) / 12.0).ceil();
                    let h = (((*hv as f64) * wh) / 12.0).ceil();

                    cr.set_source_rgb(
                        UI_GUI_BG_CLR.0,
                        UI_GUI_BG_CLR.1,
                        UI_GUI_BG_CLR.2);
                    cr.rectangle(x, y, w, h);
                    cr.fill();

                    cr.set_line_width(UI_BORDER_WIDTH);
                    cr.rectangle(x, y, w, h);
                    cr.set_source_rgb(
                        UI_BORDER_CLR.0,
                        UI_BORDER_CLR.1,
                        UI_BORDER_CLR.2);
                    cr.stroke();

                    for el in elements.iter() {
                        match el {
                            UIInput::Knob { id, label, xv, yv } => {
                                let xe = x + (((*xv as f64) * w) / 12.0).floor();
                                let ye = y + (((*yv as f64) * h) / 12.0).floor();

                                let az = self.cache.draw_knob_bg(cr, xe, ye, label);
                                self.add_active_zone(*id, az);

                                let hover =
                                    if let Some(hover_zone) = self.hover_zone {
                                        if hover_zone.id == *id {
                                            true
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    };

                                let val = self.get_element_value(*id) as f64;
                                let val_str = format!("{:4.2}", val);
                                // TODO: cache strings in a cache structure with inner
                                //       mutability and pass around Rc<String>!
                                self.cache.draw_knob_data(cr, xe, ye, hover, val, &val_str);
                            },
                            UIInput::KnobSmall { id, label, xv, yv } => {
                            },
                        }
                    }
                    //d// println!("DRAW CONTAINER {},{},{},{}", x, y, w, h);
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
