mod segmented_knob;
mod button;
mod painting;
mod draw_cache;
mod element;
mod util;
mod graph;
pub mod constants;
pub mod protocol;

use std::rc::Rc;
use std::cell::RefCell;

use crate::ui::element::*;
use crate::ui::painting::{ActiveZone, HLStyle};
use crate::ui::draw_cache::{DrawCache};
use crate::ui::protocol::{UIMsg, UICmd, UIPos, UIKnobData, UIProviderHandle,
                          UILayout, UIBtnData, UIInput, UIValueSpec, UIGraphValueSource};
use crate::ui::constants::*;

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

#[derive(Debug, Clone, Copy)]
pub enum UIEvent {
    MousePosition(f64, f64),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    WindowClose,
}

#[derive(Debug, Clone, Copy)]
enum InputMode {
    None,
    ValueDrag  { zone: ActiveZone, orig_pos: (f64, f64) },
    SelectMod  { zone: ActiveZone },
    ToggleBtn  { zone: ActiveZone },
    SetDefault { zone: ActiveZone },
}

impl InputMode {
    fn id(&self) -> usize {
        match self {
            InputMode::None                    => IMAGINARY_MAX_ID,
            InputMode::ValueDrag  { zone, .. } => zone.id,
            InputMode::SelectMod  { zone, .. } => zone.id,
            InputMode::ToggleBtn  { zone, .. } => zone.id,
            InputMode::SetDefault { zone, .. } => zone.id,
        }
    }
}

pub struct UI {
    ui_handle:      UIProviderHandle,

    font:           Option<cairo::FontFace>,

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

impl UIGraphValueSource for UI {
    fn param_value(&mut self, idx: usize) -> f32 {
        self.get_element_value(idx)
    }
}

impl UI {
    pub fn new(ui_handle: UIProviderHandle) -> Self {
        let mut this =
            Self {
                ui_handle,
                layout:             Rc::new(RefCell::new(vec![])),
                window_size:        (0.0, 0.0),
                zones:              vec![],
                cache:              DrawCache::new(),
                element_values:     vec![],
                value_specs:        vec![],
                hover_zone:         None,
                drag_tmp_value:     None,
                last_mouse_pos:     (0.0, 0.0),
                needs_redraw_flag:  true,
                input_mode:         InputMode::None,
                font:               None,
            };
        this.init_draw_cache();
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

    pub fn handle_client_command(&mut self) {
        while let Ok(cmd) = self.ui_handle.rx.try_recv() {
            match cmd {
                UICmd::Define(layout) => {
                    self.layout = Rc::new(RefCell::new(layout));
                    self.queue_redraw();
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
        if let InputMode::ValueDrag{ zone, orig_pos } = self.input_mode {
            let xd = self.last_mouse_pos.0 - orig_pos.0;
            let yd = self.last_mouse_pos.1 - orig_pos.1;
            let mut distance = xd + -yd; // (xd * xd).sqrt() (yd * yd).sqrt();

            let steps = distance / 10.0;

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
                        self.ui_handle.tx
                            .send(UIMsg::ValueChanged {
                                id:            id,
                                value:         self.get_element_value(id),
                                single_change: false,
                            })
                            .expect("Sending works");
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
                                    };
                                self.recalc_drag_value();

                                self.ui_handle.tx
                                    .send(UIMsg::ValueChangeStart {
                                        id: id, value: self.get_element_value(id)
                                    })
                                    .expect("Sending works");
                                self.queue_redraw();

                                //d// println!("drag start! {:?}", self.input_mode);
                            },
                            painting::AZ_TOGGLE => {
                                self.input_mode =
                                    InputMode::ToggleBtn {
                                        zone:     self.hover_zone.unwrap(),
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

                        self.ui_handle.tx
                            .send(UIMsg::ValueChangeEnd { id: id, value: v })
                            .expect("Sending works");

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
                                self.ui_handle.tx
                                    .send(UIMsg::ValueChanged {
                                        id:            zone.id,
                                        value:         0.0,
                                        single_change: true
                                    })
                                    .expect("Sending works");
                            }
                        }

                        self.queue_redraw();
                    },
                    InputMode::SelectMod { zone, .. } => {
                        //d// println!("MOD SELECT RELEASE");
                        if let Some(hover_zone) = self.hover_zone {
                            if hover_zone.id == zone.id {
                                self.set_element_value(zone.id, 0.0);

                                self.ui_handle.tx
                                    .send(UIMsg::ValueChanged {
                                        id:            zone.id,
                                        value:         0.0,
                                        single_change: true
                                    })
                                    .expect("Sending works");
                                self.queue_redraw();

                            } else if self.is_mod_target_value(zone.id, hover_zone.id) {
                                //d// println!("****** MOD TARGET FOR {} FOUND: {}",
                                //d//          zone.id,
                                //d//          hover_zone.id);
                                self.set_element_value(zone.id, hover_zone.id as f32);
                                self.ui_handle.tx
                                    .send(UIMsg::ValueChanged {
                                        id:            zone.id,
                                        value:         hover_zone.id as f32,
                                        single_change: true
                                    })
                                    .expect("Sending works");
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
                }

                //d// println!("BUTTON RELEASE: {:?} @{:?} / {:?}",
                //d//          btn, self.last_mouse_pos, self.input_mode);

                self.input_mode     = InputMode::None;
                self.drag_tmp_value = None;
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

    fn get_prev_toggle_value(&self, id: usize) -> f32 {
        let cur = self.get_element_value(id);
        let new_x = cur + self.value_specs[id].fine(1.0) as f32;

        if new_x < -0.001 { 1.0 }
        else              { new_x }
    }

    fn get_next_toggle_value(&self, id: usize) -> f32 {
        let cur = self.get_element_value(id);
        let new_x = cur + self.value_specs[id].coarse(1.0) as f32;

        if (new_x - 1.0) > 0.001 { 0.0 }
        else                     { new_x }
    }

    fn is_mod_target_value(&self, mod_id: usize, id: usize) -> bool {
        self.value_specs[mod_id].v2v(id as f64) > 0.5
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

    fn add_active_zone(&mut self, id: usize, mut az: ActiveZone) {
        az.id = id;
        self.zones.push(az);
    }

    fn draw_element(&mut self,
        cr: &cairo::Context,
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
            cr.set_line_width(1.0);
            cr.set_source_rgb(1.0, 0.0, 1.0);
            cr.rectangle(xe, ye, size.0, size.1);
            cr.stroke();
        }

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

        let highlight =
            match self.input_mode {
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

        let val     = self.get_element_value(id) as f64;
        let val_str = self.get_formatted_value(id);
        self.cache.draw_data(cr, xe, ye, cache_idx as usize,
                             highlight, element_data, val, &val_str);
    }

    fn layout_container(&mut self, cr: &cairo::Context, crect: Rect, rows: &Vec<Vec<UIInput>>) {
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
                            cr, &el_rect, pos.alignment(),
                            btn_data,
                            ElementType::Button);
                    },
                    UIInput::Knob(knob_data) => {
                        self.draw_element(
                            cr, &el_rect, pos.alignment(),
                            knob_data,
                            ElementType::Knob);
                    },
                    UIInput::KnobSmall(knob_data) => {
                        self.draw_element(
                            cr, &el_rect, pos.alignment(),
                            knob_data,
                            ElementType::KnobSmall);
                    },
                    UIInput::KnobHuge(knob_data) => {
                        self.draw_element(
                            cr, &el_rect, pos.alignment(),
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
                            cr, &el_rect, pos.alignment(),
                            graph_data,
                            match el {
                                UIInput::Graph(_)      => ElementType::Graph,
                                UIInput::GraphSmall(_) => ElementType::GraphSmall,
                                _                      => ElementType::GraphHuge,
                            });
                    },
                    UIInput::Container(_, childs) => {
                        let crect = el_rect;
                        self.layout_container(cr, crect, childs);
                    },
                }
            }

            row_offs = min_row_offs;
        }
    }

    pub fn draw(&mut self, cr: &cairo::Context) {
        let (ww, wh) = self.window_size;

        if let Some(ff) = self.font.as_ref() {
            cr.set_font_face(ff);
        } else {
            let ff = cairo::FontFace::toy_create(
                "serif",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Normal);
            cr.set_font_face(&ff);
            self.font = Some(ff);
        }

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

                    cr.rectangle(
                        x - UI_BORDER_WIDTH,
                        y - UI_BORDER_WIDTH,
                        w + 2.0 * UI_BORDER_WIDTH,
                        h + 2.0 * UI_BORDER_WIDTH);
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

                    if label.len() > 0 {
                        // Draw container title with some padding in relation
                        // to the border size.
                        self.cache.draw_container_label(
                            cr, x + UI_BORDER_WIDTH, y + UI_BORDER_WIDTH, label);
                    }

                    self.layout_container(cr, crect, rows);

                    //d// println!("DRAW CONTAINER {},{},{},{}", x, y, w, h);
                },
            }
        }

        self.needs_redraw_flag = false;
    }
}
