use kickmessvst;
use kickmessvst::plug_ui::{Element, ElementState, PlugUI, PlugUIPainter, UIPainter};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use kickmessvst::proc::Param;

pub trait UIParameterProvider : Send + Sync {
    fn get(&self, p: Param) -> f32;
    fn set(&self, p: Param, v: f32);
    fn start_edit(&self, p: Param) {}
    fn end_edit(&self, p: Param) {}
}

struct UITestParams {
    p: Arc<Mutex<Vec<f32>>>,
}

impl UITestParams {
    fn new() -> Self {
        Self {
            p: Arc::new(Mutex::new(vec![0.1, 0.25, 0.75])),
        }
    }
}

impl UIParameterProvider for UITestParams {
    fn set(&self, p: Param, v: f32) {
        match p {
            Param::Freq1  => { self.p.lock().unwrap()[0] = v; },
            Param::Freq2  => { self.p.lock().unwrap()[1] = v; },
            Param::Decay1 => { self.p.lock().unwrap()[2] = v; },
            _ => (),
        }
    }

    fn get(&self, p: Param) -> f32 {
        match p {
            Param::Freq1  => self.p.lock().unwrap()[0],
            Param::Freq2  => self.p.lock().unwrap()[1],
            Param::Decay1 => self.p.lock().unwrap()[2],
            _ => 0.0,
        }
    }
}


struct UI {
    needs_redraw: u32,
    params: Arc<dyn UIParameterProvider>,
}

impl UI {
    fn new() -> Self {
        Self {
            needs_redraw: 0,
            params: Arc::new(UITestParams::new()),
        }
    }
}

impl PlugUI for UI {
    fn needs_redraw(&mut self) -> bool {
        self.needs_redraw < 3
    }

    fn get_labels(&mut self, idx: usize) -> Vec<String> {
        vec![
            String::from("Start"),
            String::from("End"),
            String::from("Note"),
        ]
    }

    fn redraw(&mut self, p: &mut PlugUIPainter) {
        let elems = [
            Element::Knob(     0, 0),
            Element::SmallKnob(1, 1),
            Element::Toggle(   2, 2),
        ];

        let states = [
            ElementState::Active(self.params.get(Param::Freq1) as f64),
            ElementState::Disabled(self.params.get(Param::Freq2) as f64),
            ElementState::Hover(self.params.get(Param::Decay1) as f64),
        ];

        p.start_redraw();
        p.paint_element_hbox("Frequency", 0, 0, &elems, &states);
        p.done_redraw();

        self.needs_redraw += 1;
    }

    fn handle_input(&mut self, p: &mut PlugUIPainter) {
    }
}


fn main() {
    let handle =
        kickmessvst::baseview::open_window(
            None,
            Box::new(UI::new()));

    handle.app_run_blocking();
}
