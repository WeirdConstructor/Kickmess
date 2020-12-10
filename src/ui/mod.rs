mod segmented_knob;
mod button;
mod painting;
mod draw_cache;
mod element;
mod util;
pub mod constants;
pub mod protocol;

use std::rc::Rc;
use std::cell::RefCell;

use crate::ui::element::*;
use crate::ui::painting::{Painter, ActiveZone};
use crate::ui::draw_cache::{DrawCache};
use crate::ui::protocol::{UIMsg, UICmd, UIPos, UIKnobData, UIProviderHandle,
                          UILayout, UIBtnData, UIInput, UIValueSpec};
use crate::ui::constants::*;

fn clamp01(x: f32) -> f32 {
    if x < 0.0 { return 0.0; }
    if x > 1.0 { return 1.0; }
    x
}

#[derive(Debug, Clone, Copy)]
pub enum ElementType {
    Knob,
    KnobSmall,
    KnobHuge,
    Button,
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
    WindowClose,
}

pub struct UI {
    ui_handle:      UIProviderHandle,

//    painter:        Painter,

    layout:         Rc<RefCell<Vec<UILayout>>>,

    element_values: Vec<f32>,
    value_specs:    Vec<UIValueSpec>,
    window_size:    (f64, f64),

    zones:          Vec<ActiveZone>,
    cache:          DrawCache,

    hover_zone:     Option<ActiveZone>,
    drag_zone:      Option<((f64, f64), ActiveZone)>,
    drag_tmp_value: Option<(usize, f64)>,
    last_mouse_pos: (f64, f64),

    needs_redraw_flag: bool,
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn calc_element_box(&self, row_offs: u8, col_offs: u8, pos: UIPos) -> (Rect, u8, u8) {
        let x = self.x + ((self.w * (col_offs     as f64)) / 12.0).round();
        let y = self.y + ((self.h * (row_offs     as f64)) / 12.0).round();
        let w =          ((self.w * (pos.col_size as f64)) / 12.0).round();
        let h =          ((self.h * (pos.row_size as f64)) / 12.0).round();

        let new_row_offs = row_offs + pos.row_size;
        let new_col_offs = col_offs + pos.col_size;

        (Rect { x, y, w, h }, new_row_offs, new_col_offs)
    }
}

impl UI {
    pub fn new(ui_handle: UIProviderHandle) -> Self {
        let mut this =
            Self {
                ui_handle,
    //            painter:        Painter::new(),
                layout:             Rc::new(RefCell::new(vec![])),
                window_size:        (0.0, 0.0),
                zones:              vec![],
                cache:              DrawCache::new(),
                element_values:     vec![],
                value_specs:        vec![],
                hover_zone:         None,
                drag_tmp_value:     None,
                drag_zone:          None,
                last_mouse_pos:     (0.0, 0.0),
                needs_redraw_flag:  true,
            };
        this.init_draw_cache();
        this
    }

    fn init_draw_cache(&mut self) {
        use crate::ui::segmented_knob::SegmentedKnob;
        use crate::ui::button::Button;

        // ElementType::Knob
        self.cache.push_element(
            Box::new(SegmentedKnob::new(
                UI_KNOB_RADIUS,
                UI_KNOB_FONT_SIZE,
                UI_KNOB_FONT_SIZE - 1.0)));

        // ElementType::KnobSmall
        self.cache.push_element(
            Box::new(SegmentedKnob::new(
                (UI_KNOB_RADIUS * 0.75).round(),
                (UI_KNOB_FONT_SIZE * 0.75).round(),
                ((UI_KNOB_FONT_SIZE - 1.0) * 0.8).round())));

        // ElementType::KnobHuge
        self.cache.push_element(
            Box::new(SegmentedKnob::new(
                (UI_KNOB_RADIUS * 1.3).round(),
                (UI_KNOB_FONT_SIZE + 2.0).round(),
                UI_KNOB_FONT_SIZE + 1.0)));

        // ElementType::Button
        self.cache.push_element(Box::new(Button::new()));

//            button: SegmentedButton::new(UI_KNOB_FONT_SIZE),
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
                UICmd::DefineValues(valspecs) => {
                    self.set_value_specs(valspecs);
                },
                UICmd::SetValues(vals) => {
                    for v in vals.iter() {
                        self.set_element_value(v.id, v.value);
                    }
                },
            }
        }
        // check ui_handle
    }

    fn hover_zone_submode(&self) -> i8 {
        if let Some(hz) = self.hover_zone { hz.subtype as i8 } else { -1 }
    }

    fn recalc_drag_value(&mut self) {
        if let Some(drag_zone) = self.drag_zone {
            let xd = self.last_mouse_pos.0 - drag_zone.0.0;
            let yd = self.last_mouse_pos.1 - drag_zone.0.1;
            let mut distance = xd + -yd; // (xd * xd).sqrt() (yd * yd).sqrt();

            let steps = distance / 10.0;

            let step_val =
                if drag_zone.1.subtype == 0 {
                    self.calc_coarse_step(drag_zone.1.id, steps)
                } else {
                    self.calc_fine_step(drag_zone.1.id, steps)
                };

            self.drag_tmp_value = Some((drag_zone.1.id, step_val));
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
                            //d// println!("handle_mouse: {},{} => Hoverzone={}",
                            //d//          x, y, zone.id);
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
                use crate::ui::painting;
                match self.hover_zone_submode() {
                    painting::AZ_COARSE_DRAG | painting::AZ_FINE_DRAG => {
                        if self.drag_zone.is_none() && self.hover_zone.is_some() {
                            self.drag_zone = Some((self.last_mouse_pos, self.hover_zone.unwrap()));
                            self.recalc_drag_value();
                            self.queue_redraw();
                            println!("drag start! {:?}", self.drag_zone);
                        } else {
                            println!("BUTTON PRESS: {:?} @{:?}", btn, self.last_mouse_pos);
                        }
                    },
                    painting::AZ_MOD_SELECT => {
                    },
                    painting::AZ_TOGGLE => {
                    },
                    _ => {}
                }
            },
            UIEvent::MouseButtonReleased(btn) => {
                self.recalc_drag_value();

                if let Some(drag_tmp_value) = self.drag_tmp_value {
                    let id = drag_tmp_value.0;
                    let v = self.get_element_value(id);
                    self.set_element_value(id, v);
                    // TODO: Send message with the virtually adjusted
                    //       value to the client!
                    self.queue_redraw();
                }

                self.drag_zone      = None;
                self.drag_tmp_value = None;

                println!("BUTTON RELEASE: {:?} @{:?}", btn, self.last_mouse_pos);
            },
            UIEvent::WindowClose => {
                self.ui_handle.tx.send(
                    UIMsg::WindowClosed).expect("Sending works");
            },
            _ => {},
        }
    }

    fn set_value_specs(&mut self, valspecs: Vec<UIValueSpec>) {
        for (i, _) in valspecs.iter().enumerate() {
            self.touch_element_value(i);
        }

        self.value_specs = valspecs;
    }

    fn calc_coarse_step(&self, id: usize, steps: f64) -> f64 {
        if id >= self.value_specs.len() {
            return steps;
        }

        self.value_specs[id].coarse(steps)
    }

    fn calc_fine_step(&self, id: usize, steps: f64) -> f64 {
        if id >= self.value_specs.len() {
            return steps;
        }

        self.value_specs[id].fine(steps)
    }

    fn get_formatted_value(&self, id: usize) -> String {
        if id >= self.value_specs.len() {
            return String::from("bad valspec id");
        }

        self.value_specs[id].fmt(self.get_element_value(id) as f64)
    }

    fn touch_element_value(&mut self, id: usize) {
        if id >= self.element_values.len() {
            self.element_values.resize(id * 2, 0.0);
        }
    }

    fn set_element_value(&mut self, id: usize, value: f32) {
        if id >= self.element_values.len() {
            self.element_values.resize(id * 2, 0.0);
        }

        self.element_values[id] = value;
    }

    fn get_element_value(&self, id: usize) -> f32 {
        if id >= self.element_values.len() {
            return 0.0;
        }

        let mut v = self.element_values[id];

        if let Some(drag_tmp_value) = self.drag_tmp_value {
            if drag_tmp_value.0 == id {
                v = (v as f64 + drag_tmp_value.1) as f32;
            }
        }

        clamp01(v)
    }

    fn add_active_zone(&mut self, id: usize, mut az: ActiveZone) {
        az.id = id;
        self.zones.push(az);
    }

    fn draw_element(&mut self,
        cr: &cairo::Context,
        rect: &Rect,
        align: i8,
        element_data: &dyn UIElementData,
        cache_idx: ElementType) {

        let size = self.cache.size_of(cache_idx as usize);

        let mut xe = rect.x;
        let mut ye = rect.y;

        match align {
            1 => { xe += rect.w - size.0; },
            0 => { xe += ((rect.w - size.0) / 2.0).round(); },
            _ => { /* left align is a nop */ },
        }

        ye += (rect.h - size.1).round();

        let id = element_data.value_id();

        let mut zones : [Option<ActiveZone>; 4] = [None; 4];
        let mut z_idx = 0;

        let az = self.cache.draw_bg(cr, xe, ye, cache_idx as usize);
        self.cache.define_active_zones(xe, ye, element_data, cache_idx as usize, &mut |az| {
            zones[z_idx] = Some(az);
            z_idx += 1;
        });

        for z in zones.into_iter() {
            if let Some(az) = z {
                self.add_active_zone(id, *az);
            }
        }

        let hover =
            if let Some(hover_zone) = self.hover_zone {
                if hover_zone.id == id {
                    true
                } else {
                    false
                }
            } else {
                false
            };

        let val     = self.get_element_value(id) as f64;
        let val_str = self.get_formatted_value(id);
        self.cache.draw_data(cr, xe, ye, cache_idx as usize,
                             hover, element_data, val, &val_str);
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
                UILayout::Container { label, xv, yv, wv, hv, rows } => {
                    let x = (((*xv as f64) * ww) / 12.0).floor();
                    let y = (((*yv as f64) * wh) / 12.0).floor();
                    let w = (((*wv as f64) * ww) / 12.0).ceil();
                    let h = (((*hv as f64) * wh) / 12.0).ceil();

                    let crect = Rect { x, y, w, h };

                    cr.rectangle(x - UI_BORDER_WIDTH, y - UI_BORDER_WIDTH, w + 2.0 * UI_BORDER_WIDTH, h + 2.0 * UI_BORDER_WIDTH);
                    cr.set_source_rgb(
                        UI_BORDER_CLR.0,
                        UI_BORDER_CLR.1,
                        UI_BORDER_CLR.2);
                    cr.fill();

                    cr.set_source_rgb(
                        UI_GUI_BG_CLR.0,
                        UI_GUI_BG_CLR.1,
                        UI_GUI_BG_CLR.2);
                    cr.rectangle(x, y, w, h);
                    cr.fill();


                    let mut row_offs     = 0;
                    for row in rows.iter() {
                        let mut col_offs = 0;

                        let mut min_row_offs = 255;
                        for el in row.iter() {
                            let pos = el.position();
                            let (el_rect, ro, co) =
                                crect.calc_element_box(row_offs, col_offs, pos);
//                            println!("CALC ELEM POS={:?} => row={},col={} => ro={},co={}",
//                                    pos,
//                                    row_offs, col_offs,
//                                    ro, co);

                            col_offs = co;

                            if ro < min_row_offs { min_row_offs = ro; }

                            match el {
                                UIInput::None(_) => {
                                    // it's just about co/ro
                                },
                                UIInput::Button(btn_data) => {
                                    self.draw_element(
                                        cr, &el_rect, pos.align,
                                        btn_data,
                                        ElementType::Button);
                                },
                                UIInput::Knob(knob_data) => {
                                    self.draw_element(
                                        cr, &el_rect, pos.align,
                                        knob_data,
                                        ElementType::Knob);
                                },
                                UIInput::KnobSmall(knob_data) => {
                                    self.draw_element(
                                        cr, &el_rect, pos.align,
                                        knob_data,
                                        ElementType::KnobSmall);
                                },
                                UIInput::KnobHuge(knob_data) => {
                                    self.draw_element(
                                        cr, &el_rect, pos.align,
                                        knob_data,
                                        ElementType::KnobHuge);
                                },
                            }
                        }

                        row_offs = min_row_offs;
                    }
                    //d// println!("DRAW CONTAINER {},{},{},{}", x, y, w, h);
                },
            }
        }

        self.needs_redraw_flag = false;
    }
}
