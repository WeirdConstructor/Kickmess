use kickmessvst;
use kickmessvst::plug_ui::{Element, ElementState, PlugUI, PlugUIPainter, UIPainter};
use std::rc::Rc;
use std::sync::Arc;

struct UI {
    needs_redraw: bool,
}

impl UI {
    fn new() -> Self {
        Self {
            needs_redraw: true,
        }
    }
}

impl PlugUI for UI {
    fn needs_redraw(&mut self) -> bool {
        self.needs_redraw
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
            ElementState::Active(0.3),
            ElementState::Disabled(0.5),
            ElementState::Hover(0.5),
        ];

        p.start_redraw();
        p.paint_element_hbox("Frequency", 0, 0, &elems, &states);
        p.done_redraw();

        self.needs_redraw = false;
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
