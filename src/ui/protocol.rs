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
    fmt: Arc<dyn Fn(f64, f64) -> String + Send + Sync>,
    coarse_step:    f64,
    fine_step:      f64,
    default:        f64,
}

impl std::fmt::Debug for UIValueSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UIValueSpec")
    }
}

impl UIValueSpec {
    pub fn new_id() -> Self {
        Self {
            fun: Arc::new(|x| x),
            fmt: Arc::new(|_, x| format!("{:4.2}", x)),
            coarse_step: 0.05,
            fine_step:   0.001,
            default:     0.0,
        }
    }

    pub fn new(fun: Arc<dyn Fn(f64) -> f64 + Send + Sync>) -> Self {
        Self {
            fun,
            fmt: Arc::new(|_, x| format!("{:4.2}", x)),
            coarse_step: 0.05,
            fine_step:   0.001,
            default:     0.0,
        }
    }

    pub fn new_toggle(targets: &[&str]) -> Self {
        let max_idx = (targets.len() as f64) - 1.0;

//        let idx_fracts : Vec<f64> =
//            targets.iter().enumerate().map(
//                |(i, p)| ((i as f64) / max_idx)).collect();

        let strings : Vec<String> =
            targets.iter().map(|p| p.to_string()).collect();

        Self {
            fun: Arc::new(move |_x| {
                0.0
            }),
            fmt: Arc::new(move |v, _| {
                let mut idx : usize = (v * max_idx).round() as usize;
                if idx > strings.len() {
                    "?".to_string()
                } else {
                    strings[idx].clone()
                }
            }),
            coarse_step: (1.0 / max_idx),
            fine_step:  -(1.0 / max_idx),
            default:     0.0,
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
            fmt: Arc::new(move |v, _| {
                for (id, s) in id_2_str_map.iter() {
                    if *id == (v.round() as usize) {
                        return s.clone()
                    }
                }

                empty_label.clone()
            }),
            coarse_step: 0.0,
            fine_step:   0.0,
            default:     0.0,
        }
    }

    pub fn new_min_max(min: f64, max: f64, width: usize, prec: usize) -> Self {
        Self {
            fun: Arc::new(move |x| min * (1.0 - x) + max * x),
            fmt: Arc::new(move |_, x| format!("{2:0$.1$}", width, prec, x)),
            coarse_step: 0.05,
            fine_step:   0.001,
            default:     0.0,
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
                        fmt: Arc<dyn Fn(f64, f64) -> String + Send + Sync>) -> Self {
        Self {
            fun,
            fmt: Arc::new(|_, x| format!("{:4.2}", x)),
            coarse_step: 0.05,
            fine_step:   0.001,
            default:     0.0,
        }
    }

    pub fn fine(&self, steps: f64) -> f64   { self.fine_step   * steps }
    pub fn coarse(&self, steps: f64) -> f64 { self.coarse_step * steps }
    pub fn v2v(&self, x: f64) -> f64        { (self.fun)(x) }
    pub fn fmt(&self, x: f64) -> String     { (self.fmt)(x, self.v2v(x)) }
    pub fn get_default(&self) -> f64        { self.default }
}

// TODO: Define default margin/padding between grid cells
#[derive(Debug, Clone, Copy)]
pub struct UIPos {
    pub col_size: u8,
    pub row_size: u8,
    pub align:    i8,
    // TODO:
    //      - vertical align
}

impl UIPos {
    pub fn center(col_size: u8, row_size: u8)  -> Self { UIPos { col_size, row_size, align:  0 } }
    pub fn left(col_size: u8, row_size: u8)    -> Self { UIPos { col_size, row_size, align: -1 } }
    pub fn right(col_size: u8, row_size: u8)   -> Self { UIPos { col_size, row_size, align:  1 } }
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
    // TODO:
    //   - Type:    LeftRight, Center
}

impl UIElementData for UIKnobData {
    fn as_knob_data(&self) -> Option<&UIKnobData> { Some(self) }
    fn value_id(&self) -> usize { self.id }
}

pub trait UIGraphValueSource {
    fn param_value(&mut self, idx: usize) -> f32;
}

#[derive(Clone)]
pub struct UIGraphData {
    pub pos:         UIPos,
    pub id:          usize,
    pub label:       String,
    pub data:        Box<std::cell::RefCell<Vec<(f32,f32)>>>,
    pub fun:         Arc<dyn Fn(&mut dyn UIGraphValueSource, &mut Vec<(f32,f32)>) + Send + Sync>,
}

impl std::fmt::Debug for UIGraphData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("UIGraphData")
    }
}

impl UIGraphData {
    pub fn new(id: usize, label: String, pos: UIPos) -> Self {
        Self {
            id,
            label,
            pos,
            data: Box::new(std::cell::RefCell::new(vec![])),
            fun: Arc::new(|src, out| {
                let samples = 40;
                for x in 0..(samples + 1) {
                    let x = x as f32 / (samples as f32);
                    out.push((
                        x,
                        ((x
                         * (4.0 * src.param_value(11) + 1.0)
                         * 2.0 * std::f32::consts::PI)
                        .sin() + 1.0) / 2.0));
                }
            }),
        }
    }
}

impl UIElementData for UIGraphData {
    fn as_graph_data(&self) -> Option<&UIGraphData> { Some(self) }
    fn value_id(&self) -> usize { self.id }
}

#[derive(Debug, Clone)]
pub enum UIInput {
    None(UIPos),
    KnobSmall(UIKnobData),
    Knob(UIKnobData),
    KnobHuge(UIKnobData),
    Button(UIBtnData),
    Graph(UIGraphData),
    //      SubContainer    (size always fills)
    //      Graph           (function for plotting, predefined set of points)
}

impl UIInput {
    pub fn none(pos: UIPos) -> Self { UIInput::None(pos) }

    pub fn position(&self) -> UIPos {
        match self {
            UIInput::None(p)                             => *p,
            UIInput::KnobSmall(UIKnobData { pos, .. })   => *pos,
            UIInput::Knob(UIKnobData { pos, .. })        => *pos,
            UIInput::KnobHuge(UIKnobData { pos, .. })    => *pos,
            UIInput::Button(UIBtnData { pos, .. })       => *pos,
            UIInput::Graph(UIGraphData { pos, .. })      => *pos,
        }
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

    pub fn graph(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::Graph(UIGraphData::new(id, label, pos))
    }

    pub fn knob_small(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::KnobSmall(UIKnobData { id, label, pos })
    }

    pub fn knob_huge(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::KnobHuge(UIKnobData { id, label, pos })
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

/// Flows from UI "user" aka the plugin aka the client
/// to the UI provider (the UI thread / window abstraction).
#[derive(Debug, Clone)]
pub enum UICmd {
    Define(Vec<UILayout>),
    DefineValues(Vec<UIValueSpec>),
    SetValues(Vec<UIInputValue>),
}

/// Flows back from the UI provider to the UI client.
/// See also `UICmd`.
#[derive(Debug, Clone)]
pub enum UIMsg {
    ValueChangeStart { id: usize, value: f32 },
    ValueChangeEnd   { id: usize, value: f32 },
    ValueChanged     { id: usize, value: f32, single_change: bool },
    WindowClosed,
}

pub struct UIClientHandle {
    pub rx: Receiver<UIMsg>,
    pub tx: Sender<UICmd>,
}

impl UIClientHandle {
    pub fn create() -> (UIClientHandle, UIProviderHandle) {
        let (tx_cl,  rx_cl)  = std::sync::mpsc::channel();
        let (tx_srv, rx_srv) = std::sync::mpsc::channel();

        (UIClientHandle {
            rx: rx_cl,
            tx: tx_srv,
         }, UIProviderHandle {
            rx: rx_srv,
            tx: tx_cl,
         })
    }
}

pub struct UIProviderHandle {
    pub rx: Receiver<UICmd>,
    pub tx: Sender<UIMsg>,
}
