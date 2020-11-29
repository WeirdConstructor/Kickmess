use std::sync::mpsc::{Sender, Receiver};

pub enum UIComponent {
}

/// Flows from UI "user" aka the plugin aka the client
/// to the UI provider (the UI thread / window abstraction).
pub enum UICmd {
    Define(Vec<UIComponent>),
    SetValue { id: usize, value: f64 },
}

/// Flows back from the UI provider to the UI client.
/// See also `UICmd`.
pub enum UIMsg {
    ValueChangeStart { id: usize, value: f64 },
    ValueChangeEnd   { id: usize, value: f64 },
    ValueChanged     { id: usize, value: f64 },
}

pub struct UIClientHandle {
    rx: Receiver<UIMsg>,
    tx: Sender<UICmd>,
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
    rx: Receiver<UICmd>,
    tx: Sender<UIMsg>,
}
