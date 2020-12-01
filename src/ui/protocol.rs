use std::sync::mpsc::{Sender, Receiver};

#[derive(Debug, Clone)]
pub struct UIInputValue {
    id:     usize,
    value:  f32,
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
    SetValues(Vec<UIInputValue>),
}

/// Flows back from the UI provider to the UI client.
/// See also `UICmd`.
#[derive(Debug, Clone)]
pub enum UIMsg {
    ValueChangeStart { id: usize, value: f64 },
    ValueChangeEnd   { id: usize, value: f64 },
    ValueChanged     { id: usize, value: f64 },
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
