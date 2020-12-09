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
    fmt: Arc<dyn Fn(f64) -> String + Send + Sync>,
    coarse_step:    f64,
    fine_step:      f64,
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
            fmt: Arc::new(|x| format!("{:4.2}", x)),
            coarse_step: 0.05,
            fine_step:   0.001,
        }
    }

    pub fn new(fun: Arc<dyn Fn(f64) -> f64 + Send + Sync>) -> Self {
        Self {
            fun,
            fmt: Arc::new(|x| format!("{:4.2}", x)),
            coarse_step: 0.05,
            fine_step:   0.001,
        }
    }

    pub fn new_min_max(min: f64, max: f64, width: usize, prec: usize) -> Self {
        Self {
            fun: Arc::new(move |x| min * (1.0 - x) + max * x),
            fmt: Arc::new(move |x| format!("{2:0$.1$}", width, prec, x)),
            coarse_step: 0.05,
            fine_step:   0.001,
        }
    }

    pub fn steps(mut self, coarse: f64, fine: f64) -> Self {
        self.coarse_step = coarse;
        self.fine_step   = fine;
        self
    }

    pub fn new_with_fmt(fun: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
                        fmt: Arc<dyn Fn(f64) -> String + Send + Sync>) -> Self {
        Self {
            fun,
            fmt: Arc::new(|x| format!("{:4.2}", x)),
            coarse_step: 0.05,
            fine_step:   0.001,
        }
    }

    pub fn fine(&self, steps: f64) -> f64   { self.fine_step   * steps }
    pub fn coarse(&self, steps: f64) -> f64 { self.coarse_step * steps }
    pub fn v2v(&self, x: f64) -> f64        { (self.fun)(x) }
    pub fn fmt(&self, x: f64) -> String     { (self.fmt)(self.v2v(x)) }
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

#[derive(Debug, Clone)]
pub struct UIBtnData {
    pub pos:         UIPos,
    pub id:          usize,
    pub label:       String,
    pub labels:      Vec<(f64, String)>,
    pub mod_mode:    bool,
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

#[derive(Debug, Clone)]
pub enum UIInput {
    None(UIPos),
    KnobSmall(UIKnobData),
    Knob(UIKnobData),
    KnobHuge(UIKnobData),
    Button(UIBtnData),
    // TODO:
    //      ToggleBtn       (2 or more choices)
    //          => Button for setting a modulation target
    //             - clicking on the mod button highlights all values (in a diff, color => new highlight mode)
    //             - clicking on a value field in that mode will set
    //               the value for the mod button to the ID of the
    //               target. => The value spec fun() for unit conversion
    //               gets as input the possible value IDs, and should return
    //               a non 0.0
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
        }
    }

    pub fn btn_2state(id: usize, label: String, on_lbl: String, off_lbl: String, pos: UIPos) -> Self {
        UIInput::Button(UIBtnData {
            id, pos, label, mod_mode: false, labels: vec![
                (0.0, off_lbl), (1.0, on_lbl)
            ] })
    }

    pub fn knob(id: usize, label: String, pos: UIPos) -> Self {
        UIInput::Knob(UIKnobData { id, label, pos })
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
    ValueChangeStart { id: usize, value: f64 },
    ValueChangeEnd   { id: usize, value: f64 },
    ValueChanged     { id: usize, value: f64 },
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
