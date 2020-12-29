// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

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

pub fn define_gui(gui_hdl: &ui::protocol::UIClientHandle) {
    gui_hdl.tx.send(UICmd::DefineValues(vec![
        UIValueSpec::new_id().help("S Freq", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
//        UIValueSpec::new_min_max_exp(5.0, 3000.0, 6, 1).steps(0.04, 0.01).help("S Freq", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
//        UIValueSpec::new_min_max_exp(5.0, 2000.0, 6, 1).steps(0.04, 0.01).help("E Freq", "END fwof ewiof ew\nfewfwiuofewoi ENDoi fewoif \nfiewfoiew ENDew jfewoij \nfwefiwfh ENDfuhi "),
        UIValueSpec::new_min_max_exp(5.0, 3000.0, 6, 1).steps(0.04, 0.01),
        UIValueSpec::new_min_max_exp(5.0, 2000.0, 6, 1).steps(0.04, 0.01),
        UIValueSpec::new_min_max_exp(5.0, 5000.0, 6, 1).steps(0.04, 0.01).help("3", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
        UIValueSpec::new_min_max(0.0, 100.0, 5, 1).steps(0.04, 0.01).help("4", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
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
        UIValueSpec::new_id(),
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
       || | Freq Env      |  | Amp Env + Rel || |                 ||
       || |_______________|  |_______________|| |_________________||
       ||                                     |                    |
       ||    O       O           O       _v__ | ____________       |
       || Length Amp Slope Amp Release  |Phas|| |Dist | Off |      |
       ||                               |____|| |-----------------||
       ||     O          O        O           | |                 ||
       ||  F. Start   F. End   F. Slope   O   | |  O       O   O  ||
       ||  [N.St]     [N.End]           Click | | Start   End Gain||
       ||_____________________________________| |_________________||
       |___________________________________________________________|
    */

    gui_hdl.tx.send(UICmd::Define(vec![
        UILayout::Container {
            label: String::from(""),
            xv: 0, yv: 0, wv: 12, hv: 12,
            rows: vec![
                vec![
                    UIInput::Tabs(UITabData {
                        pos: UIPos::center(12, 12),
                        id: 13,
                        labels: vec![
                            String::from("Main"),
                            String::from("About"),
                        ],
                        childs: vec![
                            vec![
                                vec![
                                    UIInput::knob(      4, String::from("Dist S."), UIPos::center(3, 12)),
                                ]
                            ],
                            vec![
                                vec![
                                    UIInput::label_mono(
r#"Kickmess - A Kick Drum Synthesizer Plugin
Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

----------------------------
You may retrieve the source code along with a full copy of
the license at one of the following locations:

- <https://github.com/WeirdConstructor/Kickmess>
- <https://m8geil.de/repo/Kickmess>
----------------------------

The DSP code that was translated from LMMS C++ to Rust and was originally
released under GNU General Public License Version 2 or any later.
The former authors were:

* Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
* Copyright (c) 2014 grejppi <grejppi/at/gmail.com>

The fonts used are:
DejaVuSerif.ttf and DejaVuSansMono.ttf under the license:

Fonts are (c) Bitstream (see below). DejaVu changes are in public domain.
Glyphs imported from Arev fonts are (c) Tavmjong Bah (see below)
"#,
                                        14.0,
                                        UIPos::left(12, 12).top()),
                                ]
                            ],
                        ]
                    })
                ],
//                    vec![
//                        UIInput::container_border(UIPos::center(12, 4), vec![ vec![
//                            UIInput::knob(      4, String::from("Dist S."), UIPos::center(3, 12)),
//                            UIInput::knob_small(5, String::from("Dist E."), UIPos::center(2, 12)),
//                            UIInput::knob_huge( 1, String::from("SFreq."),  UIPos::center(3, 12)),
//                            UIInput::btn_toggle(10, String::from("Mod2"),   UIPos::center(4, 12)),
//                        ], ]),
//                    ],
//                    vec![
//                        UIInput::container_border(UIPos::center(12, 4), vec![ vec![
//                            UIInput::knob(      1, String::from("SFreq."),   UIPos::left(3, 12).bottom()),
//                            UIInput::knob_small(1, String::from("SFreq."),   UIPos::left(2, 12).bottom()),
//                            UIInput::knob_huge( 1, String::from("SFreq."),   UIPos::left(3, 12).bottom()),
//                            UIInput::btn_drag_value(7, String::from("Mod3"), UIPos::left(4, 12).bottom()),
//                        ], ]),
//                    ],
            ],
        },
    ])).expect("sending GUI definition works");
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
        define_gui(self.gui_hdl.as_ref().unwrap());
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
