// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use crate::ui::element::UIElementData;

#[derive(Debug, Clone)]
pub struct UIInputValue {
    pub id:     usize,
    pub value:  f32,
}

#[derive(Clone)]
pub struct UIValueSpec {
    fun: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
    fmt: Arc<dyn Fn(f64, f64, &mut std::io::Write) -> bool + Send + Sync>,
    parse: Arc<dyn Fn(&str) -> Option<f64> + Send + Sync>,
    coarse_step:    f64,
    fine_step:      f64,
    default:        f64,
    help_name:      String,
    help_text:      String,
}

impl std::fmt::Debug for UIValueSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UIValueSpec")
    }
}

fn clamp01(x: f64) -> f64 {
    if x < 0.0 { return 0.0; }
    if x > 1.0 { return 1.0; }
    x
}

impl UIValueSpec {
    pub fn new_id() -> Self {
        Self {
            fun: Arc::new(|x| x),
            fmt: Arc::new(|_, x, writer| write!(writer, "{:4.2}", x).is_ok()),
            parse: Arc::new(|s| {
                match s.parse::<f64>() {
                    Ok(v) => Some(v),
                    Err(_) => None,
                }
            }),
            coarse_step: 0.05,
            fine_step:   0.01,
            default:     0.0,
            help_name:   "".to_string(),
            help_text:   "".to_string(),
        }
    }

    pub fn new(fun: Arc<dyn Fn(f64) -> f64 + Send + Sync>) -> Self {
        Self {
            fun,
            fmt: Arc::new(|_, x, writer| write!(writer, "{:4.2}", x).is_ok()),
            parse: Arc::new(|s| {
                match s.parse::<f64>() {
                    Ok(v) => Some(v),
                    Err(_) => None,
                }
            }),
            coarse_step: 0.05,
            fine_step:   0.01,
            default:     0.0,
            help_name:   "".to_string(),
            help_text:   "".to_string(),
        }
    }

    pub fn help(mut self, name: &str, text: &str) -> Self {
        self.help_name = name.to_string();
        self.help_text = text.to_string();
        self
    }

    pub fn new_toggle(targets: &[&str]) -> Self {
        let num_targets = targets.len() as f64;
        let increment   = 1.0 / num_targets;

        let strings : Vec<String> =
            targets.iter().map(|p| p.to_string()).collect();

        Self {
            fun: Arc::new(move |x| {
                let prev = x < -0.0001;
                let x = x.abs();
                let x = if x < 0.0001 { increment * 0.5 } else { x };
                if prev {
                    if x - increment <= 0.0 {
                        1.0 - (increment * 0.5)
                    } else {
                        x - increment
                    }
                } else {
                    if x + increment >= 1.0 {
                        increment * 0.5
                    } else {
                        x + increment
                    }
                }
            }),
            fmt: Arc::new(move |v, _, writer| {
                let mut idx : usize = (v * num_targets).floor() as usize;
                if idx >= strings.len() {
                    write!(writer, "?").is_ok()
                } else {
                    write!(writer, "{}", strings[idx]).is_ok()
                }
            }),
            parse: Arc::new(|_| None),
            coarse_step: 0.0,
            fine_step:   0.0,
            default:     increment,
            help_name:   "".to_string(),
            help_text:   "".to_string(),
        }
    }

    pub fn toggle_next(&self, v: f32) -> f32 {
        (self.fun)(v as f64) as f32
    }

    pub fn toggle_prev(&self, v: f32) -> f32 {
        (self.fun)(-1.0 * v as f64) as f32
    }

    pub fn get_help_tuple(&self) -> Option<(String, String)> {
        if self.help_text.len() > 0 {
            Some((self.help_name.clone(), self.help_text.clone()))
        } else {
            None
        }
    }

    pub fn new_mod_target_list(targets: &[(usize, &str)], empty_label: &str) -> Self {
        let possible_ids : Vec<usize> =
            targets.iter().map(|p| p.0).collect();

        let id_2_str_map : Vec<(usize, String)> =
            targets.iter().map(|p| (p.0, p.1.to_string())).collect();

        let empty_label = empty_label.to_string();

        Self {
            fun: Arc::new(move |x| {
                for id in possible_ids.iter() {
                    if *id == (x.round() as usize) {
                        return 1.0;
                    }
                }

                0.0
            }),
            fmt: Arc::new(move |v, _, writer| {
                for (id, s) in id_2_str_map.iter() {
                    if *id == (v.round() as usize) {
                        return write!(writer, "{}", s).is_ok();
                    }
                }

                write!(writer, "{}", empty_label).is_ok()
            }),
            parse: Arc::new(|_| None),
            coarse_step: 0.0,
            fine_step:   0.0,
            default:     0.0,
            help_name:   "".to_string(),
            help_text:   "".to_string(),
        }
    }

    pub fn new_min_max_exp(min: f64, max: f64, width: usize, prec: usize) -> Self {
        let (min_p, max_p) = (min, max);

        Self {
            fun: Arc::new(move |x| min * (1.0 - (x * x)) + max * (x * x)),
            fmt: Arc::new(move |_, x, writer| write!(writer, "{2:0$.1$}", width, prec, x).is_ok()),
            parse: Arc::new(move |s| {
                match s.parse::<f64>() {
                    Ok(v) => {
                        let v = if v > max_p { max_p } else { v };
                        let v = if v < min_p { min_p } else { v };
                        Some(clamp01((((v - min_p) / (max_p - min_p)).abs()).sqrt()))
                    },
                    Err(_) => None,
                }
            }),
            coarse_step: 0.05,
            fine_step:   0.001,
            default:     0.0,
            help_name:   "".to_string(),
            help_text:   "".to_string(),
        }
    }

    pub fn new_min_max(min: f64, max: f64, width: usize, prec: usize) -> Self {
        let (min_p, max_p) = (min, max);

        Self {
            fun: Arc::new(move |x| min * (1.0 - x) + max * x),
            fmt: Arc::new(move |_, x, writer| write!(writer, "{2:0$.1$}", width, prec, x).is_ok()),
            parse: Arc::new(move |s| {
                match s.parse::<f64>() {
                    Ok(v) => Some(clamp01((v - min_p) / (max_p - min_p).abs())),
                    Err(_) => None,
                }
            }),
            coarse_step: 0.05,
            fine_step:   0.001,
            default:     0.0,
            help_name:   "".to_string(),
            help_text:   "".to_string(),
        }
    }

    pub fn default(mut self, default: f64) -> Self {
        self.default = default;
        self
    }

    pub fn steps(mut self, coarse: f64, fine: f64) -> Self {
        self.coarse_step = coarse;
        self.fine_step   = fine;
        self
    }

    pub fn new_with_fmt(fun: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
                        fmt: Arc<dyn Fn(f64, f64, &mut std::io::Write) + Send + Sync>) -> Self {
        Self {
            fun,
            fmt: Arc::new(|_, x, writer| write!(writer, "{:4.2}", x).is_ok()),
            parse: Arc::new(|_| None),
            coarse_step: 0.05,
            fine_step:   0.001,
            default:     0.0,
            help_name:   "".to_string(),
            help_text:   "".to_string(),
        }
    }

    pub fn fine(&self, steps: f64) -> f64   { self.fine_step   * steps }
    pub fn coarse(&self, steps: f64) -> f64 { self.coarse_step * steps }
    pub fn v2v(&self, x: f64) -> f64        { (self.fun)(x) }
    pub fn parse(&self, s: &str) -> Option<f64> { (self.parse)(s) }
    pub fn get_default(&self) -> f64        { self.default }
    pub fn fmt(&self, x: f64, writer: &mut std::io::Write) -> bool { (self.fmt)(x, self.v2v(x), writer) }
}

// TODO: Define default margin/padding between grid cells
#[derive(Debug, Clone, Copy)]
pub struct UIPos {
    pub col_size: u8,
    pub row_size: u8,
    pub align:    i8,
    pub valign:   i8,
    // TODO:
    //      - vertical align
}

impl UIPos {
    pub fn center(col_size: u8, row_size: u8)  -> Self { UIPos { col_size, row_size, align:  0, valign: 0 } }
    pub fn left(col_size: u8, row_size: u8)    -> Self { UIPos { col_size, row_size, align: -1, valign: 0 } }
    pub fn right(col_size: u8, row_size: u8)   -> Self { UIPos { col_size, row_size, align:  1, valign: 0 } }
    pub fn middle(mut self) -> Self { self.valign = 0;  self }
    pub fn top(mut self) -> Self    { self.valign = -1; self }
    pub fn bottom(mut self) -> Self { self.valign = 1;  self }
    pub fn alignment(&self) -> (i8, i8) {
        (self.align, self.valign)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UIBtnMode {
    Toggle,
    ModTarget,
    ValueDrag,
}

#[derive(Debug, Clone)]
pub struct UIBtnData {
    pub pos:         UIPos,
    pub id:          usize,
    pub label:       String,
    pub mode:        UIBtnMode,
}

impl UIElementData for UIBtnData {
    fn as_btn_data(&self) -> Option<&UIBtnData> { Some(self) }
    fn value_id(&self) -> usize { self.id }
}

#[derive(Debug, Clone)]
pub struct UIKnobData {
    pub pos:         UIPos,
    pub id:          usize,
    pub label:       String,
}

impl UIElementData for UIKnobData {
    fn as_knob_data(&self) -> Option<&UIKnobData> { Some(self) }
    fn value_id(&self) -> usize { self.id }
}

pub trait UIGraphValueSource {
    fn param_value(&mut self, idx: usize) -> f64;
}

#[derive(Clone)]
pub struct UIGraphData {
    pub pos:         UIPos,
    pub id:          usize,
    pub label:       String,
    pub data:        Box<std::cell::RefCell<Vec<(f64,f64)>>>,
    pub fun:         Arc<dyn Fn(usize, &mut dyn UIGraphValueSource, &mut Vec<(f64,f64)>) + Send + Sync>,
}

impl std::fmt::Debug for UIGraphData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UIGraphData")
    }
}

impl UIGraphData {
    pub fn new(id: usize, label: String, pos: UIPos, fun: Arc<dyn Fn(usize, &mut dyn UIGraphValueSource, &mut Vec<(f64, f64)>) + Send + Sync>) -> Self {
        Self {
            id,
            label,
            pos,
            data: Box::new(std::cell::RefCell::new(vec![])),
            fun,
        }
    }
}

impl UIElementData for UIGraphData {
    fn as_graph_data(&self) -> Option<&UIGraphData> { Some(self) }
    fn value_id(&self) -> usize { self.id }
}

#[derive(Debug, Clone)]
pub struct UITabData {
    pub pos:         UIPos,
    pub id:          usize,
    pub labels:      Vec<String>,
    pub childs:      Vec<Vec<Vec<UIInput>>>,
}

#[derive(Debug, Clone)]
pub enum UIInput {
    None(UIPos),
    Container(UIPos, Vec<Vec<UIInput>>, bool, f32),
    Label(UIPos, f32, String),
    LabelMono(UIPos, f32, String),
    Tabs(UITabData),
    KnobSmall(UIKnobData),
    Knob(UIKnobData),
    KnobHuge(UIKnobData),
    Button(UIBtnData),
    Graph(UIGraphData),
    GraphHuge(UIGraphData),
    GraphSmall(UIGraphData),
}

impl UIInput {
    pub fn none(pos: UIPos) -> Self { UIInput::None(pos) }

    pub fn position(&self) -> UIPos {
        match self {
            UIInput::None(p)                             => *p,
            UIInput::Container(p, _, _, _)               => *p,
            UIInput::Label(p, _, _)                      => *p,
            UIInput::LabelMono(p, _, _)                  => *p,
            UIInput::Tabs(UITabData { pos, .. })         => *pos,
            UIInput::KnobSmall(UIKnobData { pos, .. })   => *pos,
            UIInput::Knob(UIKnobData { pos, .. })        => *pos,
            UIInput::KnobHuge(UIKnobData { pos, .. })    => *pos,
            UIInput::Button(UIBtnData { pos, .. })       => *pos,
            UIInput::Graph(UIGraphData { pos, .. })      => *pos,
            UIInput::GraphHuge(UIGraphData { pos, .. })  => *pos,
            UIInput::GraphSmall(UIGraphData { pos, .. }) => *pos,
        }
    }

    pub fn label(label: &str, font_size: f32, pos: UIPos) -> Self {
        UIInput::Label(pos, font_size, label.to_string())
    }

    pub fn label_mono(label: &str, font_size: f32, pos: UIPos) -> Self {
        UIInput::LabelMono(pos, font_size, label.to_string())
    }

    pub fn lines_mono(label: &str, font_size: f32, pos: UIPos) -> Self {
        UIInput::lines_intern(label, font_size, pos, false, true)
    }

    pub fn lines_border_mono(label: &str, font_size: f32, pos: UIPos) -> Self {
        UIInput::lines_intern(label, font_size, pos, true, true)
    }

    pub fn lines(label: &str, font_size: f32, pos: UIPos) -> Self {
        UIInput::lines_intern(label, font_size, pos, false, false)
    }

    pub fn lines_border(label: &str, font_size: f32, pos: UIPos) -> Self {
        UIInput::lines_intern(label, font_size, pos, true, false)
    }

    fn lines_intern(label: &str, font_size: f32, pos: UIPos, border: bool, mono: bool) -> Self {
        let lines : Vec<String> = label.split("\n").map(|s| s.to_string()).collect();
        let mut childs = vec![];
        let height = (12.0 / (lines.len() as f32)).ceil() as u8;
        for l in lines {
            childs.push(vec![
                if mono {
                    UIInput::LabelMono(UIPos::left(12, height), font_size, l.to_string())
                } else {
                    UIInput::Label(UIPos::left(12, height), font_size, l.to_string())
                }
            ]);
        }
        UIInput::Container(pos, childs, border, 1.0)
    }

    pub fn btn_drag_value(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::Button(UIBtnData { id, pos, label, mode: UIBtnMode::ValueDrag })
    }

    pub fn btn_toggle(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::Button(UIBtnData { id, pos, label, mode: UIBtnMode::Toggle })
    }

    pub fn btn_mod_target(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::Button(UIBtnData { id, pos, label, mode: UIBtnMode::ModTarget })
    }

    pub fn knob(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::Knob(UIKnobData { id, label, pos })
    }

    pub fn graph(id: usize, label: String, pos: UIPos, fun: Arc<dyn Fn(usize, &mut dyn UIGraphValueSource, &mut Vec<(f64, f64)>) + Send + Sync>) -> Self {
        UIInput::Graph(UIGraphData::new(id, label, pos, fun))
    }

    pub fn graph_huge(id: usize, label: String, pos: UIPos, fun: Arc<dyn Fn(usize, &mut dyn UIGraphValueSource, &mut Vec<(f64, f64)>) + Send + Sync>) -> Self {
        UIInput::GraphHuge(UIGraphData::new(id, label, pos, fun))
    }

    pub fn graph_small(id: usize, label: String, pos: UIPos, fun: Arc<dyn Fn(usize, &mut dyn UIGraphValueSource, &mut Vec<(f64, f64)>) + Send + Sync>) -> Self {
        UIInput::GraphSmall(UIGraphData::new(id, label, pos, fun))
    }

    pub fn knob_small(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::KnobSmall(UIKnobData { id, label, pos })
    }

    pub fn knob_huge(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::KnobHuge(UIKnobData { id, label, pos })
    }

    pub fn container_border(pos: UIPos, size_factor: f32, childs: Vec<Vec<UIInput>>) -> Self {
        UIInput::Container(pos, childs, true, size_factor)
    }

    pub fn container(pos: UIPos, size_factor: f32, childs: Vec<Vec<UIInput>>) -> Self {
        UIInput::Container(pos, childs, false, size_factor)
    }
}

#[derive(Debug, Clone)]
pub enum UILayout {
    Container {
        label:    String,
        xv:       u8,
        yv:       u8,
        wv:       u8,
        hv:       u8,
        rows:     Vec<Vec<UIInput>>,
    },
}

pub trait UI {
    fn define_layout(&mut self, layout: Vec<UILayout>);
    fn define_value_spec(&mut self, spec: Vec<UIValueSpec>);
    fn set_values(&mut self, vals: &[UIInputValue]);
    fn set_version(&mut self, version_label: &'static str);
}

pub trait UIController : Send + Sync {
    fn init(&self, ui: &mut dyn UI);
    fn value_change_start(&self, ui: &mut dyn UI, id: usize, value: f32) { }
    fn value_change(&self, ui: &mut dyn UI, id: usize, value: f32, single_change: bool) { }
    fn value_change_stop(&self, ui: &mut dyn UI, id: usize, value: f32) { }
    fn window_closed(&self, ui: &mut dyn UI) {}
    fn pre_frame(&self, ui: &mut dyn UI) {}
    fn post_frame(&self, ui: &mut dyn UI) {}
}
