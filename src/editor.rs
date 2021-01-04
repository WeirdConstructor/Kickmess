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

pub const WINDOW_WIDTH:  i32 = 700;
pub const WINDOW_HEIGHT: i32 = 440;

pub(crate) struct KickmessEditor {
//    view:      Option<Box<PuglView<PuglUI>>>,
    host:       HostCallback,
    params:     Arc<KickmessVSTParams>,
    gui_hdl:    Option<ui::protocol::UIClientHandle>,
}

pub fn define_gui(gui_hdl: &ui::protocol::UIClientHandle) {
    let mut values = vec![];
    values.resize(15, UIValueSpec::new_id());

    let id_s_freq    = 0;
    let id_e_freq    = 1;
    let id_f_env_rel = 2;
    let id_env_slope = 6;
    let id_f_slope   = 7;
    let id_n_s_freq  = 9;
    let id_n_e_freq  = 10;
    let id_env_rel   = 11;
    let id_click     = 12;
    let id_main_tab  = 13;
    let id_lic_tab   = 14;

    values[id_n_s_freq] = UIValueSpec::new_toggle(&[ "Off", "On" ]);
    values[id_n_e_freq] = UIValueSpec::new_toggle(&[ "Off", "On" ]);

//        UIValueSpec::new_id().help("S Freq", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
////        UIValueSpec::new_min_max_exp(5.0, 3000.0, 6, 1).steps(0.04, 0.01).help("S Freq", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
////        UIValueSpec::new_min_max_exp(5.0, 2000.0, 6, 1).steps(0.04, 0.01).help("E Freq", "END fwof ewiof ew\nfewfwiuofewoi ENDoi fewoif \nfiewfoiew ENDew jfewoij \nfwefiwfh ENDfuhi "),
//        UIValueSpec::new_min_max_exp(5.0, 3000.0, 6, 1).steps(0.04, 0.01),
//        UIValueSpec::new_min_max_exp(5.0, 2000.0, 6, 1).steps(0.04, 0.01),
//        UIValueSpec::new_min_max_exp(5.0, 5000.0, 6, 1).steps(0.04, 0.01).help("3", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
//        UIValueSpec::new_min_max(0.0, 100.0, 5, 1).steps(0.04, 0.01).help("4", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
//        UIValueSpec::new_min_max(0.0, 100.0, 5, 1).steps(0.04, 0.01),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_mod_target_list(&[
//            (1, "Start (Hz)"),
//            (2, "End (Hz)"),
//            (3, "Length (ms)"),
//        ], "?"),
//        UIValueSpec::new_toggle(&[ "Off", "On", "Left", "Right" ]),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),
//        UIValueSpec::new_id(),

    gui_hdl.tx.send(UICmd::DefineValues(values)).expect("mpsc ok");

    let id_s_freq_f     = id_s_freq;
    let id_ae_f_env_rel = id_f_env_rel;
    let id_ae_f_slope   = id_f_slope;
    let id_ae_s_freq    = id_s_freq;
    let id_ae_e_freq    = id_e_freq;
    let f_env_fun =
        Arc::new(move |_id: usize, src: &mut dyn UIGraphValueSource, out: &mut Vec<(f64, f64)>| {
            let min_x = 0.2;
            let max_x =
                min_x + (1.0 - min_x) * src.param_value(id_ae_f_env_rel).sqrt();
            let slope = src.param_value(id_ae_f_slope).max(0.01);

            let (sign, y_offs) =
                if src.param_value(id_ae_s_freq) - src.param_value(id_ae_e_freq) < 0.0 {
                    (-1.0, -1.0)
                } else {
                    (1.0, 0.0)
                };

            let samples = 80;

            for x in 0..(samples + 1) {
                let x = max_x * (x as f64 / (samples as f64));
                out.push(
                    (x,
                     y_offs
                     + (1.0 - sign * (x / max_x).powf(slope))));
            }
        });

    let id_s_freq_f     = id_s_freq;
    let id_ae_f_env_rel = id_f_env_rel;
    let id_ae_env_slope = id_env_slope;
    let id_ae_s_freq    = id_s_freq;
    let id_ae_e_freq    = id_e_freq;
    let amp_env_fun =
        Arc::new(move |_id: usize, src: &mut dyn UIGraphValueSource, out: &mut Vec<(f64, f64)>| {
            let slope = src.param_value(id_ae_env_slope).max(0.01);
            let min_x = 0.2;
            let max_x =
                min_x + (1.0 - min_x) * src.param_value(id_ae_f_env_rel).sqrt();

            let samples = 80;

            for x in 0..(samples + 1) {
                let x = max_x * (x as f64 / (samples as f64));
                out.push(
                    (x, (1.0 - (x / max_x).powf(slope))));
            }
        });

    let id_ph_click = id_click;
    let phase_fun =
        Arc::new(move |_id: usize, src: &mut dyn UIGraphValueSource, out: &mut Vec<(f64, f64)>| {
            let samples = 80;

            for x in 0..(samples + 1) {
                let x = x as f64 / (samples as f64);
                out.push((
                    x,
                    (((x + src.param_value(id_ph_click))
                       * 2.0 * std::f64::consts::PI).sin() + 1.0) / 2.0));
            }
        });

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
                        id: id_main_tab,
                        labels: vec![
                            String::from("Main"),
                            String::from("About"),
                        ],
                        childs: vec![
                            vec![
                                vec![
                                    UIInput::graph_huge(
                                        0,
                                        String::from("Amp Env"),
                                        UIPos::center(3, 4).bottom(),
                                        amp_env_fun.clone()),
                                    UIInput::knob(
                                        id_f_env_rel,
                                        String::from("Length (ms)"),
                                        UIPos::center(2, 4).bottom()),
                                    UIInput::knob(
                                        id_env_slope,
                                        String::from("Env Slope"),
                                        UIPos::center(2, 4).bottom()),
                                    UIInput::knob(
                                        id_env_rel,
                                        String::from("Release (ms)"),
                                        UIPos::center(2, 4).bottom()),
                                ],
                                vec![
                                    UIInput::graph_huge(
                                        0,
                                        String::from("Freq. Env"),
                                        UIPos::center(3, 4).bottom(),
                                        f_env_fun.clone()),
                                    UIInput::knob(
                                        id_s_freq,
                                        String::from("Start (Hz)"),
                                        UIPos::center(2, 4).bottom()),
                                    UIInput::knob(
                                        id_e_freq,
                                        String::from("End (Hz)"),
                                        UIPos::center(2, 4).bottom()),
                                    UIInput::knob(
                                        id_f_slope,
                                        String::from("Slope"),
                                        UIPos::center(2, 4).bottom()),
                                ],
                                vec![
                                    UIInput::container_border(
                                        UIPos::left(3, 4).bottom(),
                                        0.8,
                                        vec![
                                            vec![
                                                UIInput::knob_small(
                                                    id_click,
                                                    String::from("Click"),
                                                    UIPos::center(6, 12).middle()),
                                                UIInput::graph_small(
                                                    0,
                                                    String::from("Click"),
                                                    UIPos::center(6, 12).middle(),
                                                    phase_fun.clone()),
                                            ],
                                        ]),
                                    UIInput::btn_toggle(
                                        id_n_s_freq,
                                        String::from("S. from Note"),
                                        UIPos::center(2, 4).middle()),
                                    UIInput::btn_toggle(
                                        id_n_e_freq,
                                        String::from("E. from Note"),
                                        UIPos::center(2, 4).middle()),
                                ],
                            ],
                            vec![
                                vec![
                                    UIInput::Tabs(UITabData {
                                        pos: UIPos::center(12, 12),
                                        id: id_lic_tab,
                                        labels: vec![
                                            String::from("Plugin"),
                                            String::from("Fonts"),
                                        ],
                                        childs: vec![
                                    vec![vec![UIInput::label_mono(
r#"Kickmess - A Kick Drum Synthesizer Plugin
Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>

The DSP code that was translated from LMMS C++ to Rust and was originally
released under GNU General Public License Version 2 or any later.
The former authors were:

* Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
* Copyright (c) 2014 grejppi <grejppi/at/gmail.com>

You may retrieve the source code along with a full copy of
the license at one of the following locations:

- <https://github.com/WeirdConstructor/Kickmess>
- <https://m8geil.de/repo/Kickmess>
"#,
                                        14.0,
                                        UIPos::left(12, 12).top())]],
                                    vec![vec![UIInput::label_mono(
r#"The fonts used are:
DejaVuSerif.ttf and DejaVuSansMono.ttf under the license:

Fonts are (c) Bitstream. DejaVu changes are in public domain.
Glyphs imported from Arev fonts are (c) Tavmjong Bah
"#,
                                        14.0,
                                        UIPos::left(12, 12).top())]],
                                    ]}),
                                ],
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

        crate::window::open_window(
            "Kickmess",
            WINDOW_WIDTH, WINDOW_HEIGHT,
            Some(parent), p_hdl);

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
