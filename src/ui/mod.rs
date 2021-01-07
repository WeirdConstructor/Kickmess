// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

mod segmented_knob;
mod button;
mod draw_cache;
mod element;
mod graph;

pub mod painting;
pub mod constants;
pub mod protocol;

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;

use crate::ui::element::*;
use crate::ui::painting::{ActiveZone, HLStyle, Painter};
use crate::ui::draw_cache::{DrawCache};
use crate::ui::protocol::{UIPos, UIKnobData,
                          UITabData, UILayout, UIBtnData, UIInput,
                          UIValueSpec, UIGraphValueSource,
                          UIInputValue, UI, UIController};
use crate::ui::constants::*;
use keyboard_types::{Key, KeyboardEvent};

const IMAGINARY_MAX_ID : usize = 9999999999;

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
    Graph,
    GraphHuge,
    GraphSmall,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone)]
pub enum UIEvent {
    MousePosition(f64, f64),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    KeyPressed(KeyboardEvent),
    KeyReleased(KeyboardEvent),
    WindowClose,
}

#[derive(Debug, Clone)]
enum InputMode {
    None,
    ValueDrag  { zone: ActiveZone, orig_pos: (f64, f64), fine_key: bool },
    SelectMod  { zone: ActiveZone },
    ToggleBtn  { zone: ActiveZone },
    SetDefault { zone: ActiveZone },
    SetValue   { zone: ActiveZone },
    InputValue { zone: ActiveZone,
                 value: String,
                 input: std::rc::Rc<std::cell::RefCell<std::io::BufWriter<Vec<u8>>>> },
    GetHelp,
}

impl InputMode {
    fn id(&self) -> usize {
        match self {
            InputMode::None                    => IMAGINARY_MAX_ID,
            InputMode::GetHelp                 => IMAGINARY_MAX_ID,
            InputMode::ValueDrag  { zone, .. } => zone.id,
            InputMode::SelectMod  { zone, .. } => zone.id,
            InputMode::ToggleBtn  { zone, .. } => zone.id,
            InputMode::SetDefault { zone, .. } => zone.id,
            InputMode::SetValue   { zone, .. } => zone.id,
            InputMode::InputValue { zone, .. } => zone.id,
        }
    }
}

pub struct WValuePlugUI {
    controller:     Arc<dyn UIController>,

    layout:         Rc<RefCell<Vec<UILayout>>>,

    element_values: Vec<f32>,
    value_specs:    Vec<UIValueSpec>,
    window_size:    (f64, f64),

    zones:          Vec<ActiveZone>,
    cache:          DrawCache,

    hover_zone:     Option<ActiveZone>,
    drag_tmp_value: Option<(usize, f64)>,
    last_mouse_pos: (f64, f64),
    input_mode:     InputMode,
    fine_drag_key_held: bool,
    help_texts:     Vec<Option<(String, String)>>,
    help_id:        Option<usize>,

    needs_redraw_flag: bool,

    version_label:  &'static str,
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn move_inside(&self, w: f64, h: f64) -> Self {
        let x = self.x.max(0.0);
        let y = self.y.max(0.0);

        let xm = x + self.w;
        let ym = y + self.h;
        let x = if xm > w { x - (xm - w) } else { x };
        let y = if ym > h { y - (ym - h) } else { y };
        Rect { x, y, w: self.w, h: self.h }
    }

    fn mul_size(&self, factor: f64) -> Self {
        let w = self.w * factor;
        let h = self.h * factor;
        let wd = (self.w - w) / 2.0;
        let hd = (self.h - h) / 2.0;

        Self {
            x: self.x + wd,
            y: self.y + hd,
            w: self.w - wd,
            h: self.h - hd,
        }
    }

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

impl UIGraphValueSource for WValuePlugUI {
    fn param_value(&mut self, idx: usize) -> f64 {
        self.get_element_value(idx) as f64
    }
}

impl UI for WValuePlugUI {
    fn define_layout(&mut self, layout: Vec<UILayout>) {
        self.layout = Rc::new(RefCell::new(layout));
        self.queue_redraw();
    }

    fn define_value_spec(&mut self, valspecs: Vec<UIValueSpec>) {
        self.set_value_specs(valspecs);
        self.queue_redraw();
    }

    fn set_values(&mut self, vals: &[UIInputValue]) {
        for v in vals.iter() {
            self.set_element_value(v.id, v.value);
        }
        self.queue_redraw();
    }

    fn set_version(&mut self, version_label: &'static str) {
        self.version_label = version_label;
        self.queue_redraw();
    }
}

impl WValuePlugUI {
    pub fn new(controller: Arc<dyn UIController>) -> Self {
        let mut this =
            Self {
                controller,
                layout:             Rc::new(RefCell::new(vec![])),
                window_size:        (0.0, 0.0),
                zones:              vec![],
                cache:              DrawCache::new(),
                element_values:     vec![],
                value_specs:        vec![],
                hover_zone:         None,
                drag_tmp_value:     None,
                fine_drag_key_held: false,
                last_mouse_pos:     (0.0, 0.0),
                needs_redraw_flag:  true,
                input_mode:         InputMode::None,
                help_id:            None,
                help_texts:         vec![],
                version_label:      "",
            };
        this.init_draw_cache();
        this.controller.clone().init(&mut this);
        this
    }

    fn init_draw_cache(&mut self) {
        use crate::ui::segmented_knob::SegmentedKnob;
        use crate::ui::button::Button;
        use crate::ui::graph::Graph;

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

        // ElementType::Graph
        self.cache.push_element(
            Box::new(Graph::new(
                UI_GRPH_W, UI_GRPH_H, UI_GRPH_FONT_SIZE)));

        // ElementType::GraphHuge
        self.cache.push_element(
            Box::new(Graph::new(
                (UI_GRPH_W * 1.5).round(),
                (UI_GRPH_H * 1.5).round(),
                UI_GRPH_FONT_SIZE + 1.0)));

        // ElementType::GraphSmall
        self.cache.push_element(
            Box::new(Graph::new(
                (UI_GRPH_W * 0.6).round(),
                (UI_GRPH_H * 0.6).round(),
                ((UI_GRPH_FONT_SIZE - 1.0) * 0.8).round())));

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

    fn hover_zone_submode(&self) -> i8 {
        if let Some(hz) = self.hover_zone { hz.subtype as i8 } else { -1 }
    }

    fn hover_highligh_for_id(&self, id: usize) -> HLStyle {
        if let Some(hover_zone) = self.hover_zone {
            if hover_zone.id == id {
                HLStyle::Hover(hover_zone.subtype as i8)
            } else {
                HLStyle::None
            }
        } else {
            HLStyle::None
        }
    }

    fn recalc_drag_value(&mut self) {
        if let InputMode::ValueDrag { zone, orig_pos, fine_key } = self.input_mode {
            let xd = self.last_mouse_pos.0 - orig_pos.0;
            let yd = self.last_mouse_pos.1 - orig_pos.1;
            let mut distance = -yd;

            let steps = if fine_key { distance / 25.0 } else { distance / 10.0 };

            let step_val =
                if zone.subtype == 0 {
                    self.calc_coarse_step(zone.id, steps)
                } else {
                    self.calc_fine_step(zone.id, steps)
                };

            self.drag_tmp_value = Some((zone.id, step_val));
        } else {
            self.drag_tmp_value = None;
        }
    }

    pub fn handle_ui_event(&mut self, ev: UIEvent) {
        match ev {
            UIEvent::MousePosition(x, y) => {
                self.last_mouse_pos = (x, y);

                match self.input_mode {
                    InputMode::ValueDrag { zone, .. } => {
                        self.recalc_drag_value();

                        let id = zone.id;
                        let value = self.get_element_value(id);
                        self.controller.clone().value_change(
                            self, id, value, false);
                    },
                    _ => {
                        self.hover_zone = None;

                        for zone in self.zones.iter() {
                            if zone.is_inside(x, y) {
                                self.hover_zone = Some(*zone);
                                //d// println!("handle_mouse: {},{} => Hoverzone={}",
                                //d//          x, y, zone.id);
                                break;
                            }
                        }
                    },
                }

                self.queue_redraw();
            },
            UIEvent::MouseButtonPressed(btn) => {
                use crate::ui::painting;

                match self.input_mode {
                    InputMode::None => {
                        if let Some(hz) = self.hover_zone {
                            match btn {
                                MouseButton::Middle => {
                                    self.input_mode = InputMode::SetDefault { zone: hz };
                                    self.queue_redraw();
                                    return;
                                },
                                MouseButton::Right => {
                                    if    self.hover_zone_submode() == painting::AZ_COARSE_DRAG
                                       || self.hover_zone_submode() == painting::AZ_FINE_DRAG {
                                        let mut buf = vec![];
                                        let mut bw = std::io::BufWriter::new(buf);
                                        self.get_formatted_value(hz.id, &mut bw);
                                        let value = 
                                            String::from_utf8(bw.into_inner().unwrap()).unwrap();
                                        self.input_mode =
                                            InputMode::InputValue {
                                                zone: hz,
                                                value: value.trim().to_string(),
                                                input:
                                                    std::rc::Rc::new(
                                                        std::cell::RefCell::new(
                                                            std::io::BufWriter::new(vec![]))),
                                            };
                                        self.queue_redraw();
                                        return;
                                    }
                                },
                                _ => {}
                            }
                        }

                        match self.hover_zone_submode() {
                            painting::AZ_COARSE_DRAG | painting::AZ_FINE_DRAG => {
                                let id = self.hover_zone.unwrap().id;

                                self.input_mode =
                                    InputMode::ValueDrag {
                                        orig_pos: self.last_mouse_pos,
                                        zone:     self.hover_zone.unwrap(),
                                        fine_key: self.fine_drag_key_held,
                                    };
                                self.recalc_drag_value();

                                let value = self.get_element_value(id);
                                self.controller.clone().value_change_start(
                                    self, id, value);
                                self.queue_redraw();

                                //d// println!("drag start! {:?}", self.input_mode);
                            },
                            painting::AZ_TOGGLE => {
                                self.input_mode =
                                    InputMode::ToggleBtn {
                                        zone: self.hover_zone.unwrap(),
                                    };
                            },
                            painting::AZ_SET_VALUE => {
                                self.input_mode =
                                    InputMode::SetValue {
                                        zone: self.hover_zone.unwrap(),
                                    };
                            },
                            _ => {
                                println!("BUTTON PRESS: {:?} @{:?}", btn, self.last_mouse_pos);
                            }
                        }
                    },
                    _ => {}
                }
            },
            UIEvent::MouseButtonReleased(btn) => {
                match self.input_mode {
                    InputMode::None => {
                        match self.hover_zone_submode() {
                            painting::AZ_MOD_SELECT => {
                                //d// println!("MOD SELECT BUTTON PRESS: {:?} @{:?}", btn, self.last_mouse_pos);
                                self.input_mode =
                                    InputMode::SelectMod {
                                        zone: self.hover_zone.unwrap()
                                    };
                                self.queue_redraw();

                                return;
                            },
                            _ => { }
                        }
                    },
                    InputMode::ValueDrag { .. } => {
                        self.recalc_drag_value();

                        let id = self.drag_tmp_value.unwrap().0;
                        let v  = self.get_element_value(id);

                        self.set_element_value(id, v);

                        self.controller.clone().value_change_stop(self, id, v);
                        self.queue_redraw();
                    },
                    InputMode::SetDefault { zone } => {
                        if let Some(hover_zone) = self.hover_zone {
                            if hover_zone.id == zone.id {
                                self.set_element_value(
                                    zone.id,
                                    self.get_element_default_value(zone.id));
                            }
                        }

                        self.queue_redraw();
                    },
                    InputMode::ToggleBtn { zone, .. } => {
                        if let Some(hover_zone) = self.hover_zone {
                            if hover_zone.id == zone.id {
                                let next =
                                    match btn {
                                        MouseButton::Left =>
                                            self.get_next_toggle_value(zone.id),
                                        MouseButton::Right =>
                                            self.get_prev_toggle_value(zone.id),
                                        _ =>
                                            self.get_element_default_value(zone.id),
                                    };

                                self.set_element_value(zone.id, next);
                                self.controller.clone().value_change(
                                    self, zone.id, next, true);
                            }
                        }

                        self.queue_redraw();
                    },
                    InputMode::SetValue { zone, .. } => {
                        if let Some(hover_zone) = self.hover_zone {
                            if hover_zone.id == zone.id {
                                self.set_element_value(zone.id, zone.set_val as f32);
                                self.controller.clone().value_change(
                                    self, zone.id, zone.set_val as f32, true);
                                self.queue_redraw();
                            }
                        }
                    },
                    InputMode::GetHelp => {
                        if let Some(hover_zone) = self.hover_zone {
                            println!("HOVERZOME: {:?}", hover_zone);
                            if let Some(_) = self.get_element_help(hover_zone.id) {
                                self.help_id = Some(hover_zone.id);
                                self.queue_redraw();
                            }
                        }
                    },
                    InputMode::SelectMod { zone, .. } => {
                        //d// println!("MOD SELECT RELEASE");
                        if let Some(hover_zone) = self.hover_zone {
                            if hover_zone.id == zone.id {
                                self.set_element_value(zone.id, 0.0);

                                self.controller.clone().value_change(
                                    self, zone.id, 0.0, true);
                                self.queue_redraw();

                            } else if self.is_mod_target_value(zone.id, hover_zone.id) {
                                //d// println!("****** MOD TARGET FOR {} FOUND: {}",
                                //d//          zone.id,
                                //d//          hover_zone.id);
                                self.set_element_value(zone.id, hover_zone.id as f32);
                                self.controller.clone().value_change(
                                    self, zone.id, hover_zone.id as f32, true);
                                self.queue_redraw();
                            } else {
                                // do not exit select modulation mode
                                return;
                            }
                        } else {
                            // do not exit select modulation mode
                            return;
                        }
                    },
                    InputMode::InputValue { .. } => {
                        // stay in input value mode
                        return;
                    },
                }

                //d// println!("BUTTON RELEASE: {:?} @{:?} / {:?}",
                //d//          btn, self.last_mouse_pos, self.input_mode);

                self.input_mode     = InputMode::None;
                self.drag_tmp_value = None;
            },
            UIEvent::KeyPressed(key_event) => {
                match key_event.key {
                    Key::Shift => { self.fine_drag_key_held = true; },
                    Key::Enter => {
                        let new_value : Option<(usize, f32)> =
                            if let InputMode::InputValue { input, zone, .. } = &self.input_mode {
                                let mut r = input.borrow_mut();
                                let s = std::str::from_utf8(r.get_ref()).unwrap();

                                match self.parse_element_value(zone.id, s) {
                                    Some(v) => Some((zone.id, v as f32)),
                                    None    => None,
                                }
                            } else {
                                None
                            };

                        if let Some((id, val)) = new_value {
                            self.set_element_value(id, val);
                            self.controller.clone().value_change(
                                self, id, val, true);
                            self.input_mode = InputMode::None;
                        }

                        self.queue_redraw();
                    },
                    Key::Backspace => {
                        if let InputMode::InputValue { input, .. } = &self.input_mode {
                            let mut r = input.borrow_mut();
                            let s = std::str::from_utf8(r.get_ref()).unwrap();
                            let len = s.chars().count();
                            if len > 0 {
                                let s : String = s.chars().take(len - 1).collect();
                                *r = std::io::BufWriter::new(s.into_bytes());
                            }
                        }

                        self.queue_redraw();
                    },
                    Key::Character(c) => {
                        if let InputMode::InputValue { input, zone, .. } = &self.input_mode {
                            use std::io::Write;

                            let mut r = input.borrow_mut();
                            let prev_value = String::from_utf8(r.get_ref().to_vec()).unwrap();

                            let c = if c == "," { ".".to_string() } else { c };
                            write!(r, "{}", c);
                            r.flush();

                            let s = std::str::from_utf8(r.get_ref()).unwrap();

                            match self.parse_element_value(zone.id, s) {
                                None => {
                                    *r = std::io::BufWriter::new(prev_value.as_bytes().to_vec());
                                },
                                _ => {}
                            }
                        }

                        self.queue_redraw();
                    },
                    _ => {
//                        if 
                    }
                }
            },
            UIEvent::KeyReleased(key_event) => {
                match key_event.key {
                    Key::Shift  => { self.fine_drag_key_held = false; },
                    Key::F1     => {
                        if let Some(_) = self.help_id {
                            self.input_mode = InputMode::None;
                            self.help_id    = None;

                        } else if let InputMode::GetHelp = self.input_mode {
                            self.input_mode = InputMode::None;
                            self.help_id    = None;

                        } else {
                            self.input_mode = InputMode::GetHelp;
                        }

                        self.queue_redraw();
                    },
                    Key::Escape => {
                        self.input_mode = InputMode::None;
                        self.help_id    = None;
                        self.queue_redraw();
                    },
                    _ => { }
                }
            },
            UIEvent::WindowClose => {
                self.controller.clone().window_closed(self);
            },
            _ => {},
        }
    }

    fn set_value_specs(&mut self, valspecs: Vec<UIValueSpec>) {
        for (i, vspec) in valspecs.iter().enumerate() {
            if i >= self.help_texts.len() {
                self.help_texts.resize(i + 1, None);
            }

            self.help_texts[i] = vspec.get_help_tuple();

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

    fn get_formatted_value(&self, id: usize, writer: &mut std::io::Write) -> bool {
        if id >= self.value_specs.len() {
            write!(writer, "bad valspec id {}", id);
            return true;
        }

        self.value_specs[id].fmt(self.get_element_value(id) as f64, writer)
    }

    fn get_prev_toggle_value(&self, id: usize) -> f32 {
        let cur = self.get_element_value(id);
        dbg!(self.value_specs[id].toggle_prev(cur))
    }

    fn get_next_toggle_value(&self, id: usize) -> f32 {
        let cur = self.get_element_value(id);
        dbg!(self.value_specs[id].toggle_next(cur))
    }

    fn is_mod_target_value(&self, mod_id: usize, id: usize) -> bool {
        self.value_specs[mod_id].v2v(id as f64) > 0.5
    }

    fn parse_element_value(&self, id: usize, s: &str) -> Option<f64> {
        self.value_specs[id].parse(s)
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

    fn get_element_default_value(&self, id: usize) -> f32 {
        if id >= self.element_values.len() {
            return 0.0;
        }

        self.value_specs[id].get_default() as f32
    }

    fn get_element_value(&self, id: usize) -> f32 {
        if id >= self.element_values.len() {
            return 0.0;
        }

        let mut v = self.element_values[id];

        if let InputMode::ValueDrag { zone, .. } = self.input_mode {
            let drag_tmp_value = self.drag_tmp_value.unwrap();
            if id == zone.id {
                v = (v as f64 + drag_tmp_value.1) as f32;
                v = clamp01(v);
            }
        }

        v
    }

    fn get_element_help(&self, id: usize) -> Option<(&str, &str)> {
        if let Some(Some((n, t))) = self.help_texts.get(id) {
            Some((n, t))
        } else {
            None
        }
    }

    fn add_active_zone(&mut self, id: usize, mut az: ActiveZone) {
        az.id = id;
        self.zones.push(az);
    }

    fn draw_element(&mut self,
        p: &mut dyn Painter,
        rect: &Rect,
        align: (i8, i8),
        element_data: &dyn UIElementData,
        cache_idx: ElementType) {

        let size = self.cache.size_of(cache_idx as usize);

        let mut xe = rect.x;
        let mut ye = rect.y;

        match align.0 {
            1 => { xe += rect.w - size.0; },
            0 => { xe += ((rect.w - size.0) / 2.0).round(); },
            _ => { /* left align is a nop */ },
        }

        match align.1 {
            1 => { ye += rect.h - size.1; },
            0 => { ye += ((rect.h - size.1) / 2.0).round(); },
            _ => { /* left align is a nop */ },
        }

        let id = element_data.value_id();

        let mut zones : [Option<ActiveZone>; 4] = [None; 4];
        let mut z_idx = 0;

        if false {
            p.rect_stroke(1.0, (1.0, 0.0, 1.0), xe, ye, size.0, size.1);
        }

        let az = self.cache.draw_bg(p, xe, ye, cache_idx as usize);
        self.cache.define_active_zones(xe, ye, element_data, cache_idx as usize, &mut |az| {
            zones[z_idx] = Some(az);
            z_idx += 1;
        });

        for z in zones.into_iter() {
            if let Some(az) = z {
                self.add_active_zone(id, *az);
            }
        }

        let highlight =
            match self.input_mode {
                InputMode::GetHelp => {
                    if let HLStyle::Hover(_) = self.hover_highligh_for_id(id) {
                        HLStyle::HoverModTarget
                    } else if let Some(_) = self.get_element_help(id) {
                        HLStyle::ModTarget
                    } else {
                        HLStyle::None
                    }
                },
                InputMode::SelectMod { zone } => {
                    if let HLStyle::Hover(_) = self.hover_highligh_for_id(id) {
                        if self.is_mod_target_value(zone.id, id) {
                            HLStyle::HoverModTarget
                        } else if zone.id == id {
                            HLStyle::HoverModTarget
                        } else {
                            HLStyle::None
                        }
                    } else {
                        if self.is_mod_target_value(zone.id, id) {
                            HLStyle::ModTarget
                        } else if zone.id == id {
                            HLStyle::ModTarget
                        } else {
                            HLStyle::None
                        }
                    }
                },
                _ => {
                    self.hover_highligh_for_id(id)
                },
            };

        let mut buf : [u8; 64] = [0_u8; 64];
        let mut bw = std::io::BufWriter::new(&mut buf[..]);
        let val    = self.get_element_value(id) as f64;

        if !self.get_formatted_value(id, &mut bw) {
            self.cache.draw_data(p, xe, ye, cache_idx as usize,
                                 highlight, element_data, val,
                                 &"write! fail");
        } else {
            self.cache.draw_data(p, xe, ye, cache_idx as usize,
                                 highlight, element_data, val,
                                 &std::str::from_utf8(bw.buffer()).unwrap());
        }
    }

    fn tac_on_tab_headers(&mut self, p: &mut dyn Painter, tab_rect: Rect,
                          labels: &[String], selected_idx: usize, id: usize) {

        let mut hover_idx = 9999;

        if let Some(hover_zone) = self.hover_zone {
            if hover_zone.id == id {
                hover_idx =
                    (hover_zone.set_val * (labels.len() as f64)).floor() as usize;
                hover_idx = hover_idx.min(labels.len() - 1);
            }
        }

        let val_inc = 1.0 / (labels.len() as f64);

        let mut hover_border = None;

        let mut lbl_x = tab_rect.x;
        for (i, lbl) in labels.iter().enumerate() {
            let mut z =
                ActiveZone::from_rect(
                    lbl_x, tab_rect.y,
                    painting::AZ_SET_VALUE,
                    (0.0, 0.0, UI_TAB_WIDTH, tab_rect.h));
            z.set_val = (val_inc * 0.5) + (i as f64) * val_inc;
            self.add_active_zone(id, z);

            if i == selected_idx {
                // Tab border
                p.rect_fill(
                    UI_BORDER_CLR,
                    lbl_x, tab_rect.y,
                    UI_TAB_WIDTH,
                    tab_rect.h);

                // Tab contrast
                p.rect_fill(
                    UI_TAB_BG_CLR,
                    lbl_x      + UI_BORDER_WIDTH,
                    tab_rect.y + UI_BORDER_WIDTH,
                    UI_TAB_WIDTH - 2.0 * UI_BORDER_WIDTH,
                    tab_rect.h - UI_BORDER_WIDTH);

                // Tab text underline
                p.path_stroke(
                    UI_TAB_DIV_WIDTH,
                    UI_TAB_DIV_CLR,
                    &mut ([
                        (lbl_x + UI_BORDER_WIDTH,                (tab_rect.y + tab_rect.h - UI_TAB_DIV_WIDTH * 0.5).round()),
                        (lbl_x + UI_TAB_WIDTH - UI_BORDER_WIDTH, (tab_rect.y + tab_rect.h - UI_TAB_DIV_WIDTH * 0.5).round())
                    ].iter().copied()), false);

            } else {
                // Tab border
                p.rect_fill(
                    UI_BORDER_CLR,
                    lbl_x, tab_rect.y,
                    UI_TAB_WIDTH,
                    tab_rect.h);

                // Tab contrast
                p.rect_fill(
                    UI_TAB_BG_CLR,
                    lbl_x      + UI_BORDER_WIDTH,
                    tab_rect.y + UI_BORDER_WIDTH,
                    UI_TAB_WIDTH - 2.0 * UI_BORDER_WIDTH,
                    tab_rect.h   - 2.0 * UI_BORDER_WIDTH);
            }

            if i != selected_idx && i == hover_idx {
                // hover text label
                p.label(
                    UI_TAB_FONT_SIZE, 0, UI_TAB_TXT_HOVER_CLR,
                    lbl_x, tab_rect.y,
                    UI_TAB_WIDTH,
                    tab_rect.h,
                    lbl);

                // remember the hover border for drawing after
                // the loop.
                let hover_pad = 1.0;
                hover_border = Some(Rect {
                    x: lbl_x      + hover_pad,
                    y: tab_rect.y + hover_pad,
                    w: UI_TAB_WIDTH - 2.0 * hover_pad,
                    h: tab_rect.h   - 2.0 * hover_pad,
                });

            } else {
                let txt_clr =
                    if i == selected_idx { UI_TAB_TXT_CLR }
                    else                 { UI_TAB_TXT2_CLR };

                // normal text label, selected and non selected
                p.label(
                    UI_TAB_FONT_SIZE, 0, txt_clr,
                    lbl_x, tab_rect.y,
                    UI_TAB_WIDTH,
                    tab_rect.h, lbl);
            }

            lbl_x += UI_TAB_WIDTH - UI_BORDER_WIDTH;
        }

        if let Some(hover_border) = hover_border {
            p.rect_stroke(
                UI_BORDER_WIDTH,
                UI_TAB_TXT_HOVER_CLR,
                hover_border.x, hover_border.y,
                hover_border.w, hover_border.h);
        }

        if let InputMode::GetHelp = self.input_mode {
            if let HLStyle::Hover(_) = self.hover_highligh_for_id(id) {
                p.rect_stroke(
                    UI_BORDER_WIDTH * 2.0,
                    UI_TXT_KNOB_HLIGHT_CLR,
                    tab_rect.x, tab_rect.y,
                    tab_rect.w, tab_rect.h);
            } else if let Some(_) = self.get_element_help(id) {
                p.rect_stroke(
                    UI_BORDER_WIDTH * 2.0,
                    UI_TXT_KNOB_HLHOVR_CLR,
                    tab_rect.x, tab_rect.y,
                    tab_rect.w, tab_rect.h);
            }
        }
    }

    fn layout_container(&mut self, p: &mut dyn Painter, border: bool, label: &str, depth: u32, crect: Rect, rows: &Vec<Vec<UIInput>>) {
        let crect =
            if border {
                let crect = Rect {
                    x: crect.x + UI_MARGIN,
                    y: crect.y + UI_MARGIN,
                    w: crect.w - 2.0 * UI_MARGIN,
                    h: crect.h - 2.0 * UI_MARGIN,
                };

                p.rect_fill(UI_BORDER_CLR,
                    crect.x - UI_BORDER_WIDTH,
                    crect.y - UI_BORDER_WIDTH,
                    crect.w + 2.0 * UI_BORDER_WIDTH,
                    crect.h + 2.0 * UI_BORDER_WIDTH);

                p.rect_fill(
                    if depth % 2 == 0 { UI_GUI_BG_CLR } else { UI_GUI_BG2_CLR },
                    crect.x,
                    crect.y,
                    crect.w,
                    crect.h);

                let crect =
                    if label.len() > 0 {
                        // Draw container title with some padding in relation
                        // to the border size.
                        self.cache.draw_container_label(
                            p, crect.x, crect.y, crect.w, label);

                        Rect {
                            x: crect.x,
                            y: crect.y + UI_ELEM_TXT_H,
                            w: crect.w,
                            h: crect.h - UI_ELEM_TXT_H
                        }
                    } else {
                        crect
                    };

                Rect {
                    x: crect.x + UI_PADDING,
                    y: crect.y + UI_PADDING,
                    w: crect.w - 2.0 * UI_PADDING,
                    h: crect.h - 2.0 * UI_PADDING,
                }
            } else {
                crect
            };

        let mut row_offs = 0;
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
                            p, &el_rect, pos.alignment(),
                            btn_data,
                            ElementType::Button);
                    },
                    UIInput::Knob(knob_data) => {
                        self.draw_element(
                            p, &el_rect, pos.alignment(),
                            knob_data,
                            ElementType::Knob);
                    },
                    UIInput::KnobSmall(knob_data) => {
                        self.draw_element(
                            p, &el_rect, pos.alignment(),
                            knob_data,
                            ElementType::KnobSmall);
                    },
                    UIInput::KnobHuge(knob_data) => {
                        self.draw_element(
                            p, &el_rect, pos.alignment(),
                            knob_data,
                            ElementType::KnobHuge);
                    },
                    UIInput::Graph(graph_data) | UIInput::GraphSmall(graph_data) | UIInput::GraphHuge(graph_data) => {
                        {
                            let mut data_buf = graph_data.data.borrow_mut();
                            data_buf.clear();
                            (graph_data.fun)(graph_data.id, self, &mut data_buf);
                        }

                        self.draw_element(
                            p, &el_rect, pos.alignment(),
                            graph_data,
                            match el {
                                UIInput::Graph(_)      => ElementType::Graph,
                                UIInput::GraphSmall(_) => ElementType::GraphSmall,
                                _                      => ElementType::GraphHuge,
                            });
                    },
                    UIInput::Label(_, font_size, label) => {
                        let crect = el_rect;
                        self.draw_text_lines(
                            p, label, *font_size as f64, UI_LBL_TXT_CLR,
                            crect.x, crect.y, crect.w, crect.h, false, None);
                    },
                    UIInput::LabelMono(_, font_size, label) => {
                        let crect = el_rect;
                        self.draw_text_lines(
                            p, label, *font_size as f64, UI_LBL_TXT_CLR,
                            crect.x, crect.y, crect.w, crect.h, true, None);
                    },
                    UIInput::Container(_, childs, next_border, size_factor) => {
                        let crect = el_rect.mul_size(*size_factor as f64);
                        self.layout_container(
                            p, *next_border, "",
                            if border { depth + 1 } else { depth },
                            crect, childs);
                    },
                    UIInput::Tabs(UITabData { id, labels, childs, .. }) => {
                        let crect = el_rect;

                        let tab_h = UI_ELEM_TXT_H + UI_PADDING;
                        let tab_rect = Rect {
                            x: UI_BORDER_WIDTH + crect.x,
                            y: UI_BORDER_WIDTH + crect.y,
                            w: crect.w,
                            h: tab_h + UI_BORDER_WIDTH,
                        };
                        let crect = Rect {
                            x: crect.x,
                            y: crect.y + tab_h,
                            w: crect.w,
                            h: crect.h - tab_h,
                        };

                        let mut selected_idx =
                            (self.get_element_value(*id)
                             * (labels.len() as f32)).floor() as usize;
                        let selected_idx = selected_idx.min(labels.len() - 1);

                        let next_depth =
                            if border { depth + 1 } else { depth };

                        self.layout_container(
                            p, true, "", next_depth,
                            crect, &childs[selected_idx]);

                        self.tac_on_tab_headers(
                            p, tab_rect, labels, selected_idx, *id);
                    },
                }
            }

            row_offs = min_row_offs;
        }
    }

    pub fn draw_text_lines(&self, p: &mut dyn Painter, s: &str, font_size: f64,
                           color: (f64, f64, f64),
                           x: f64, mut y: f64, w: f64, h: f64,
                           mono: bool, title: Option<&str>) {
        let y_increment = p.font_height(font_size as f32, mono) as f64;

        if let Some(title) = title {
            p.label_mono(
                1.5 * font_size, 0, color,
                x, y, w, UI_ELEM_TXT_H, title);

            y += 2.0 * y_increment;
        }

        for line in s.split("\n") {
            if mono {
                p.label_mono(
                    font_size, -1, color, x, y, w, UI_ELEM_TXT_H, line);
            } else {
                p.label(
                    font_size, -1, color, x, y, w, UI_ELEM_TXT_H, line);
            }
            y += y_increment;
        }

        if let Some(_) = title {
            p.label_mono(font_size, 0, color,
                x,
                h - 2.0 * (UI_MARGIN + UI_BORDER_WIDTH + UI_PADDING),
                w, UI_ELEM_TXT_H,
                "Press <Escape> or <F1> to exit help");
        }
    }

    pub fn pre_frame(&mut self) {
        let ctrl = self.controller.clone();
        ctrl.pre_frame(self);
    }

    pub fn post_frame(&mut self) {
        self.controller.clone().post_frame(self);
    }

    pub fn draw(&mut self, p: &mut dyn Painter) {
        let (ww, wh) = self.window_size;

        p.rect_fill(UI_GUI_BG_CLR, 0.0, 0.0, ww, wh);

        self.zones.clear();

        let layout = self.layout.clone();

        for layout in layout.borrow_mut().iter() {
            match layout {
                UILayout::Container { label, xv, yv, wv, hv, rows } => {
                    let x = (((*xv as f64) * ww) / 12.0).floor();
                    let y = (((*yv as f64) * wh) / 12.0).floor();
                    let w = (((*wv as f64) * ww) / 12.0).ceil();
                    let h = (((*hv as f64) * wh) / 12.0).ceil();

                    let x = x + UI_BORDER_WIDTH;
                    let y = y + UI_BORDER_WIDTH;
                    let w = w - 2.0 * UI_BORDER_WIDTH;
                    let h = h - 2.0 * UI_BORDER_WIDTH;

                    let crect = Rect { x, y, w, h };

                    self.layout_container(p, label.len() > 0, label, 0, crect, rows);

                    //d// println!("DRAW CONTAINER {},{},{},{}", x, y, w, h);
                },
            }
        }

        if let Some(help_id) = self.help_id {
            p.rect_fill(UI_GUI_BG_CLR, 0.0, 0.0, ww, wh);
            p.rect_stroke(
                UI_BORDER_WIDTH,
                UI_BORDER_CLR,
                UI_MARGIN, UI_MARGIN,
                ww - 2.0 * UI_MARGIN,
                wh - 2.0 * UI_MARGIN);

            if let Some((name, txt)) = self.get_element_help(help_id) {
                let y_increment = UI_ELEM_TXT_H;
                let y = UI_MARGIN + UI_BORDER_WIDTH + UI_PADDING;
                let x = UI_MARGIN + UI_BORDER_WIDTH + UI_PADDING;

                self.draw_text_lines(
                    p, txt, UI_HELP_FONT_SIZE, UI_HELP_TXT_CLR,
                    x, y, ww, wh, true, Some(name));
            }
        }

        if let InputMode::InputValue { zone, value, input } = &self.input_mode {
            let height = p.font_height(16.0, true) as f64;

            let zone_center =
                (zone.x + zone.w / 2.0,
                 zone.y + zone.h / 2.0);

            let edit_area = Rect {
                x: zone_center.0 - UI_INPUT_BOX_W / 2.0,
                y: zone_center.1,
                w: UI_INPUT_BOX_W,
                h: height * 3.0,
            };

            let edit_area = edit_area.move_inside(ww, wh);

            p.rect_fill(UI_GUI_BG_CLR, edit_area.x, edit_area.y, edit_area.w, edit_area.h);
            p.rect_stroke(
                UI_BORDER_WIDTH,
                UI_BORDER_CLR,
                edit_area.x + UI_MARGIN,
                edit_area.y + UI_MARGIN,
                edit_area.w - 2.0 * UI_MARGIN,
                edit_area.h - 2.0 * UI_MARGIN);

            let old_val_area = Rect {
                x: edit_area.x + UI_MARGIN * 2.0,
                y: edit_area.y + UI_MARGIN * 2.0,
                w: UI_INPUT_BOX_W - 4.0 * UI_MARGIN,
                h: height,
            };
            p.label_mono(
                UI_INPUT_BOX_FONT_SIZE * 0.9,
                1,
                UI_LBL_TXT_CLR,
                old_val_area.x,
                old_val_area.y,
                UI_INPUT_BOX_W / 2.0 - UI_MARGIN * 2.0,
                old_val_area.h,
                "Old:");

            p.label_mono(
                UI_INPUT_BOX_FONT_SIZE * 0.9,
                -1,
                UI_LBL_TXT_CLR,
                old_val_area.x + UI_INPUT_BOX_W / 2.0,
                old_val_area.y,
                old_val_area.w,
                old_val_area.h,
                value);

            let input_area = Rect {
                x: edit_area.x + UI_MARGIN * 2.0,
                y: edit_area.y + edit_area.h - (height + UI_MARGIN * 2.0),
                w: UI_INPUT_BOX_W - 4.0 * UI_MARGIN,
                h: height,
            };

            p.rect_fill(
                UI_LBL_BG_CLR,
                input_area.x, input_area.y, input_area.w, input_area.h);
            p.label_mono(UI_INPUT_BOX_FONT_SIZE, 0, UI_LBL_TXT_CLR,
                input_area.x, input_area.y, input_area.w, input_area.h,
                &std::str::from_utf8(input.borrow().get_ref()).unwrap().trim());
        }

        if self.version_label.len() > 0 {
            p.label_mono(
                UI_VERSION_FONT_SIZE, 1, UI_LBL_TXT_CLR,
                ww - 50.0, 0.0, 40.0, UI_ELEM_TXT_H,
                self.version_label);
        }

        self.needs_redraw_flag = false;
    }
}
