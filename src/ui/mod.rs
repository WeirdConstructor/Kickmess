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

fn clamp01(x: f32) -> f32 {
    if x < 0.0 { return 0.0; }
    if x > 1.0 { return 1.0; }
    x
}

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
    drag_zone:      Option<((f64, f64), ActiveZone)>,
    drag_tmp_value: Option<(usize, f64)>,
    last_mouse_pos: (f64, f64),

    needs_redraw_flag: bool,
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
            drag_tmp_value:     None,
            drag_zone:          None,
            last_mouse_pos:     (0.0, 0.0),
            needs_redraw_flag:  false,
        }
    }

    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw_flag
    }

    pub fn queue_redraw(&mut self) {
        self.needs_redraw_flag = true;
    }

    pub fn set_window_size(&mut self, w: f64, h: f64) {
        self.window_size = (w, h);
    }

    pub fn handle_client_command(&mut self) {
        while let Ok(cmd) = self.ui_handle.rx.try_recv() {
            match cmd {
                UICmd::Define(layout) => {
                    self.layout = Rc::new(RefCell::new(layout));
                    self.queue_redraw();
                    println!("CLIENT EVENT: LAYOUT!");
                },
                UICmd::SetValues(_) => {
                },
            }
        }
        // check ui_handle
    }

    fn recalc_drag_value(&mut self) {
        if let Some(drag_zone) = self.drag_zone {
            let xd = self.last_mouse_pos.0 - drag_zone.0.0;
            let yd = self.last_mouse_pos.1 - drag_zone.0.1;
            let mut distance = xd + yd; // (xd * xd).sqrt() (yd * yd).sqrt();

            self.drag_tmp_value = Some((drag_zone.1.id, distance / 100.0));
        } else {
            self.drag_tmp_value = None;
        }
    }

    pub fn handle_ui_event(&mut self, ev: UIEvent) {
        match ev {
            UIEvent::MousePosition(x, y) => {
                self.last_mouse_pos = (x, y);

                if self.drag_zone.is_none() {
                    self.hover_zone = None;

                    for zone in self.zones.iter() {
                        if zone.is_inside(x, y) {
                            self.hover_zone = Some(*zone);
                            println!("handle_mouse: {},{} => Hoverzone={}",
                                     x, y, zone.id);
                            break;
                        }
                    }
                } else {
                    self.recalc_drag_value();
                    // TODO: Send message with the virtually adjusted
                    //       value to the client!
                    println!("SENDBACK VALUE CHANGE: {:?}",
                        self.drag_tmp_value);
                }

                self.queue_redraw();
            },
            UIEvent::MouseButtonPressed(btn) => {
                if self.drag_zone.is_none() && self.hover_zone.is_some() {
                    self.drag_zone = Some((self.last_mouse_pos, self.hover_zone.unwrap()));
                    self.recalc_drag_value();
                    self.queue_redraw();
                    println!("drag start! {:?}", self.drag_zone);
                } else {
                    println!("BUTTON PRESS: {:?} @{:?}", btn, self.last_mouse_pos);
                }
            },
            UIEvent::MouseButtonReleased(btn) => {
                // TODO: If drag zone, then apply the value change!
                self.recalc_drag_value();

                if let Some(drag_tmp_value) = self.drag_tmp_value {
                    let id = drag_tmp_value.0;
                    let v = self.get_element_value(id);
                    self.set_element_value(id, v);
                    self.queue_redraw();
                }
                self.drag_zone      = None;
                self.drag_tmp_value = None;
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

    fn set_element_value(&mut self, id: usize, value: f32) {
        if id >= self.element_values.len() {
            self.element_values.resize(id * 2, 0.0);
        }

        self.element_values[id] = value;
    }

    fn get_element_value(&mut self, id: usize) -> f32 {
        if id >= self.element_values.len() {
            self.element_values.resize(id * 2, 0.0);
        }

        let mut v = self.element_values[id];

        if let Some(drag_tmp_value) = self.drag_tmp_value {
            if drag_tmp_value.0 == id {
                v = (v as f64 + drag_tmp_value.1) as f32;
                println!("DRAGOFF {}", drag_tmp_value.1);
            }
        }

        clamp01(v)
    }

    fn add_active_zone(&mut self, id: usize, mut az: ActiveZone) {
        az.id = id;
        self.zones.push(az);
    }

    pub fn draw(&mut self, cr: &cairo::Context) {
        let (ww, wh) = self.window_size;

        let ff = cairo::FontFace::toy_create(
            "serif",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal);
        cr.set_font_face(&ff);

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

                                let val     = self.get_element_value(*id) as f64;
                                let val_str = format!("{:4.2}", val);
                                // TODO: cache strings in a cache structure with inner
                                //       mutability and pass around Rc<String>!
                                // TODO: Cache inside draw_knob_data, pass a callback for
                                //       string formatting, so that we only allocate new strings
                                //       when actually drawing new content.
                                //       The cache inside draw_knob_data should check hover and val
                                //       for differences! => Otherwise redraw old content.
                                //       Use rounded int xe/ye as key for cache lookup!
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

        self.needs_redraw_flag = false;
    }
}
