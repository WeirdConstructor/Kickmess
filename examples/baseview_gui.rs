use kickmessvst;
use kickmessvst::ui;
use kickmessvst::ui::protocol::*;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use kickmessvst::proc::Param;

/*

UI Code Architecture:

Baseview / PUGL as Event source and Display loop:

    - on_frame() { ui.draw(); }
    - on_event(event) { ui.handle(event); }

The plugin defining which graphical components are where,
and which "id" they have:

    ui.add_hbox([Knobs with id], [Connections])

The UI handles hover state changes.

The plugin defining how value changes are interpreted:

    for e in ui.recv_event() {
        match e {
            ValueStart(ui_id)        => {},
            ValueEnd(ui_id)          => {},
            ValueEnter(ui_id, new_value01) => {
                model.change(ui2param(ui_id), new_value01);
            },
            Value(ui_id, offs_value01) => {
                model.change(
                    ui2param(ui_id),
                    model.get(ui2param(ui_id)) + offs_value * 10.0);
                ui.send_update_value(ui_id, model.get(ui2param(ui_id)));
            },
        }
    }

UI is therefore:
    - handling events
    - drawing knobs and converting the values to human readable
      values according to predefined conversions (big enum :)
    - receiving new value updates
    - drawing/updating the screen
    - following things are down now:
        - the UI thread owns the UI
        - communication with the UI happens via two way mpsc
            - the plugin gets a PluginUIHandle
            - the view implementor (baseview / pugl) owns the UI

Further abstraction:
    - A class that knows how to draw the UI definition with Cairo
    - Receives:
        - UI description
        - Interpreted values (strings)

*/

//pub trait UIParameterProvider : Send + Sync {
//    fn get(&self, p: Param) -> f32;
//    fn set(&self, p: Param, v: f32);
//    fn start_edit(&self, p: Param) {}
//    fn end_edit(&self, p: Param) {}
//}
//
//struct UITestParams {
//    p: Arc<Mutex<Vec<f32>>>,
//}
//
//impl UITestParams {
//    fn new() -> Self {
//        Self {
//            p: Arc::new(Mutex::new(vec![0.1, 0.25, 0.75])),
//        }
//    }
//}
//
//impl UIParameterProvider for UITestParams {
//    fn set(&self, p: Param, v: f32) {
//        match p {
//            Param::Freq1  => { self.p.lock().unwrap()[0] = v; },
//            Param::Freq2  => { self.p.lock().unwrap()[1] = v; },
//            Param::Decay1 => { self.p.lock().unwrap()[2] = v; },
//            _ => (),
//        }
//    }
//
//    fn get(&self, p: Param) -> f32 {
//        match p {
//            Param::Freq1  => self.p.lock().unwrap()[0],
//            Param::Freq2  => self.p.lock().unwrap()[1],
//            Param::Decay1 => self.p.lock().unwrap()[2],
//            _ => 0.0,
//        }
//    }
//}

//impl PlugUI for UI {
//    fn needs_redraw(&mut self) -> bool {
//        self.needs_redraw < 3
//    }
//
//    fn get_labels(&mut self, idx: usize) -> Vec<String> {
//        vec![
//            String::from("Start"),
//            String::from("End"),
//            String::from("Note"),
//        ]
//    }
//
//    fn redraw(&mut self, p: &mut PlugUIPainter) {
//        let elems = [
//            Element::Knob(     0, 0),
//            Element::SmallKnob(1, 1),
//            Element::Toggle(   2, 2),
//        ];
//
//        let states = [
//            ElementState::Active(self.params.get(Param::Freq1) as f64),
//            ElementState::Disabled(self.params.get(Param::Freq2) as f64),
//            ElementState::Hover(self.params.get(Param::Decay1) as f64),
//        ];
//
//        p.start_redraw();
//        p.paint_element_hbox("Frequency", 0, 0, &elems, &states);
//        p.done_redraw();
//
//        self.needs_redraw += 1;
//    }
//
//    fn handle_input(&mut self, p: &mut PlugUIPainter) {
//        self.needs_redraw = 0;
//    }
//}


fn main() {
    let (cl_hdl, p_hdl) = ui::protocol::UIClientHandle::create();

    let (handle, runner) = kickmessvst::baseview::open_window(None, p_hdl);

    cl_hdl.tx.send(UICmd::Define(vec![
        UILayout::Container {
            label: String::from("Test"),
            xv: 1,
            yv: 1,
            wv: 10,
            hv: 10,
            elements: vec![
                UIInput::Knob { label: String::from("SFreq."), id: 1, xv: 0, yv: 0, },
                UIInput::Knob { label: String::from("EFreq."), id: 2, xv: 6, yv: 0, },
                UIInput::Knob { label: String::from("Noise"), id: 3, xv: 0, yv: 4, },
                UIInput::Knob { label: String::from("SDist"), id: 4, xv: 6, yv: 4, },
                UIInput::Knob { label: String::from("EDist."), id: 5, xv: 0, yv: 8, },
                UIInput::Knob { label: String::from("F Slope"), id: 6, xv: 6, yv: 8, },
                UIInput::Knob { label: String::from("Env Slope."), id: 7, xv: 3, yv: 0, },
                UIInput::Knob { label: String::from("SFreq."), id: 8, xv: 3, yv: 4, },
            ],
        },
    ])).expect("mpsc ok");

     // TODO: Send VALUES!

    std::thread::spawn(move || {
        while let Ok(msg) = cl_hdl.rx.recv_timeout(
            std::time::Duration::from_millis(1000)) {
            println!("MSG FROM UI: {:?}", msg);
        }
    });

    runner.app_run_blocking();
}
