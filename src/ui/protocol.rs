use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;

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

#[derive(Debug, Clone)]
pub enum UIInput {
    KnobSmall { xv: u8, yv: u8, id: usize, label: String },
    Knob      { xv: u8, yv: u8, id: usize, label: String },
}

#[derive(Debug, Clone)]
pub enum UILayout {
    Container {
        label:    String,
        xv:       u8,
        yv:       u8,
        wv:       u8,
        hv:       u8,
        elements: Vec<UIInput>
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
