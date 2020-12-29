use crate::KickmessVSTParams;
use vst::editor::Editor;
use std::sync::Arc;
use std::rc::Rc;
use vst::plugin::{HostCallback};
use vst::host::Host;

use crate::ui::protocol::*;
use crate::ui::constants::*;
use crate::ui::{UI, UIEvent};
use crate::ui;

const WINDOW_WIDTH:  i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

pub(crate) struct KickmessEditor {
//    view:      Option<Box<PuglView<PuglUI>>>,
    host:       HostCallback,
    params:     Arc<KickmessVSTParams>,
    gui_hdl:    Option<ui::protocol::UIClientHandle>,
}

impl KickmessEditor {
    pub(crate) fn new(host: HostCallback, params: Arc<KickmessVSTParams>) -> Self {
        Self {
//            view: None,
            host,
            params,
            gui_hdl: None,
        }
    }

    fn define_gui(&self) {
        self.gui_hdl.as_ref().unwrap().tx.send(UICmd::DefineValues(vec![
            UIValueSpec::new_id(),
            UIValueSpec::new_min_max_exp(5.0, 3000.0, 6, 1).steps(0.04, 0.01),
            UIValueSpec::new_min_max_exp(5.0, 2000.0, 6, 1).steps(0.04, 0.01),
            UIValueSpec::new_min_max_exp(5.0, 5000.0, 6, 1).steps(0.04, 0.01),
            UIValueSpec::new_min_max(0.0, 100.0, 5, 1).steps(0.04, 0.01),
            UIValueSpec::new_min_max(0.0, 100.0, 5, 1).steps(0.04, 0.01),
            UIValueSpec::new_id(),
            UIValueSpec::new_id(),
            UIValueSpec::new_id(),
            UIValueSpec::new_mod_target_list(&[
                (1, "Start (Hz)"),
                (2, "End (Hz)"),
                (3, "Length (ms)"),
            ], "?"),
            UIValueSpec::new_toggle(&[ "Off", "On", "Left", "Right" ]),
            UIValueSpec::new_id(),
            UIValueSpec::new_id(),
            UIValueSpec::new_id(),
            UIValueSpec::new_id(),
        ])).expect("mpsc ok");

        /* ________________
           | MAIN | About |
           |======|-------|--------------------------------------------|
           ||-------------------------------------| |-----------------||
           || Osc                                 | |   O     O       ||
           || |---------------|  |---------------|| |                 ||
           || |               |  |               || |  Gain Noise     ||
           || |  Waveform     |  |[N.St] [N.End] || |                 ||
           || |_______________|  |_______________|| |_________________||
           ||                                     |                    |
           ||    O       O           O          O | ____________       |
           || Length Amp Slope Amp Release  Click | |Dist | Off |      |
           ||                                     | |-----------------||
           ||     O          O        O           | |                 ||
           ||  F. Start   F. End   F. Slope       | |  O       O   O  ||
           ||                                     | | Start   End Gain||
           ||_____________________________________| |_________________||
           |___________________________________________________________|
        */

        self.gui_hdl.as_ref().unwrap().tx.send(UICmd::Define(vec![
            UILayout::Container {
                label: String::from("Test GUI"),
                xv: 0, yv: 0, wv: 7, hv: 12,
                rows: vec![
                    vec![
                        UIInput::container_border(UIPos::center(12, 4), vec![ vec![
                                UIInput::knob(      1, String::from("Start (Hz)"),  UIPos::right(3, 12)),
                                UIInput::knob_small(2, String::from("End (Hz)"),    UIPos::right(2, 12)),
                                UIInput::knob_huge( 3, String::from("Length (ms)"), UIPos::right(3, 12)),
                                UIInput::btn_mod_target(9, String::from("Mod1"),    UIPos::right(4, 12)),
                        ], ]),
                    ],
                    vec![
                        UIInput::container_border(UIPos::center(12, 4), vec![ vec![
                            UIInput::knob(      4, String::from("Dist S."), UIPos::center(3, 12)),
                            UIInput::knob_small(5, String::from("Dist E."), UIPos::center(2, 12)),
                            UIInput::knob_huge( 1, String::from("SFreq."),  UIPos::center(3, 12)),
                            UIInput::btn_toggle(10, String::from("Mod2"),   UIPos::center(4, 12)),
                        ], ]),
                    ],
                    vec![
                        UIInput::container_border(UIPos::center(12, 4), vec![ vec![
                            UIInput::knob(      1, String::from("SFreq."),   UIPos::left(3, 12).bottom()),
                            UIInput::knob_small(1, String::from("SFreq."),   UIPos::left(2, 12).bottom()),
                            UIInput::knob_huge( 1, String::from("SFreq."),   UIPos::left(3, 12).bottom()),
                            UIInput::btn_drag_value(7, String::from("Mod3"), UIPos::left(4, 12).bottom()),
                        ], ]),
                    ],
                ],
            },
        ])).expect("sending GUI definition works");
    }
}

impl Editor for KickmessEditor {
    fn size(&self) -> (i32, i32) {
        (WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
    }

    fn position(&self) -> (i32, i32) {
        (0, 0)
    }

    fn open(&mut self, parent: *mut std::ffi::c_void) -> bool {
        let (cl_hdl, p_hdl) = ui::protocol::UIClientHandle::create();

        let runner =
            crate::window::open_window(
                "Kickmess",
                WINDOW_WIDTH, WINDOW_HEIGHT,
                Some(parent), p_hdl);
        std::thread::spawn(move || {
            runner.unwrap().app_run_blocking();
        });

        self.gui_hdl = Some(cl_hdl);
        self.define_gui();

        true
    }

    fn is_open(&mut self) -> bool {
        self.gui_hdl.is_some()
    }

    fn idle(&mut self) {
        let mut closed = false;

        if let Some(gui_hdl) = self.gui_hdl.as_mut() {
            while let Ok(msg) = gui_hdl.rx.try_recv() {
                println!("MSG FROM UI: {:?}", msg);
                match msg {
                    UIMsg::ValueChangeStart { id, value } => {
                        if let Some(af) = self.params.params.get(id) {
                            af.set(value);
                            self.host.begin_edit(id as i32);
                            self.host.automate(id as i32, value);
                        }
                    },
                    UIMsg::ValueChanged { id, value, single_change } => {
                        if let Some(af) = self.params.params.get(id) {
                            af.set(value);
                            self.host.automate(id as i32, value);
                        }
                    },
                    UIMsg::ValueChangeEnd { id, value } => {
                        if let Some(af) = self.params.params.get(id) {
                            af.set(value);
                            self.host.automate(id as i32, value);
                            self.host.end_edit(id as i32);
                        }
                    },
                    UIMsg::WindowClosed => {
                        closed = true;
                        break;
                    },
                    _ => {},
                }
            }
        }

        if closed {
            self.gui_hdl = None;
        }

    }

    fn close(&mut self) {
//        self.view.as_mut().unwrap().as_mut().handle().close_request()
    }
}
