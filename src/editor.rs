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
use crate::ui;

pub const WINDOW_WIDTH:  i32 = 700;
pub const WINDOW_HEIGHT: i32 = 440;

pub(crate) struct KickmessEditorController {
    host:    HostCallback,
    params:  Arc<KickmessVSTParams>,
    is_open: std::sync::atomic::AtomicBool,
    close_request: std::sync::atomic::AtomicBool,
}

pub(crate) struct KickmessEditor {
    controller: Arc<KickmessEditorController>,
}

impl KickmessEditorController {
    pub fn request_close(&self) {
        self.close_request.store(true, std::sync::atomic::Ordering::Relaxed)
    }
    pub fn is_still_open(&self) -> bool {
        self.is_open.load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl UIController for KickmessEditorController {
    fn init(&self, ui: &mut dyn UI) {
        self.is_open.store(true, std::sync::atomic::Ordering::Relaxed);
        define_gui(&self.params.ps, ui);

        for (i, p) in self.params.params.iter().enumerate() {
            ui.set_values(
                &[UIInputValue {
                    id: i,
                    value: p.get(),
                }]);
        }
    }

    fn window_closed(&self, _ui: &mut dyn UI) {
        self.is_open.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    fn pre_frame(&self, ui: &mut dyn UI) {
        use crate::proc::ParamProvider;

        while let Some(id) = self.params.dirty_params.pop() {
            ui.set_values(
                &[UIInputValue {
                    id: id,
                    value: self.params.param(id)
                }]);
        }
    }

    fn value_change_start(&self, ui: &mut dyn UI, id: usize, value: f32) {
        if let Some(af) = self.params.params.get(id) {
            af.set(value);
            self.host.begin_edit(id as i32);
            //d// println!("START AUTOM {}: {}", id, value);
            self.host.automate(id as i32, value);
        }
    }

    fn value_change(&self, ui: &mut dyn UI, id: usize, value: f32, single_change: bool) {
        if let Some(af) = self.params.params.get(id) {
            af.set(value);
            if single_change { self.host.begin_edit(id as i32); }
            println!("AUTOM {}: {}", id, value);
            self.host.automate(id as i32, value);
            if single_change { self.host.end_edit(id as i32); }
        }
    }

    fn value_change_stop(&self, ui: &mut dyn UI, id: usize, value: f32) {
        if let Some(af) = self.params.params.get(id) {
            af.set(value);
            //d// println!("STOP AUTOM {}: {}", id, value);
            self.host.automate(id as i32, value);
            self.host.end_edit(id as i32);
        }
    }
}

pub fn define_gui(ps: &crate::ParamSet, gui: &mut dyn ui::protocol::UI) {
    let mut values = vec![];
    values.resize(17, UIValueSpec::new_id());

    let id_s_freq    = 0;
    let id_e_freq    = 1;
    let id_f_env_rel = 2;
    let id_d_start   = 3;
    let id_d_end     = 4;
    let id_gain      = 5;
    let id_env_slope = 6;
    let id_f_slope   = 7;
    let id_noise     = 8;
    let id_n_s_freq  = 9;
    let id_n_e_freq  = 10;
    let id_env_rel   = 11;
    let id_click     = 12;
    let id_dist_on   = 13;

    let id_main_tab  = 15;
    let id_lic_tab   = 16;

    for i in 0..ps.param_count() {
        values[i] = ps.definition(i).unwrap().to_ui_value_spec();
    }
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

    gui.define_value_spec(values);

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

    let oscillator_params =
        UIInput::container_border(UIPos::center(8, 12), 1.0,
            vec![
                vec![
                    UIInput::graph_huge(
                        0,
                        String::from("Amp Env"),
                        UIPos::center(5, 4).bottom(),
                        amp_env_fun.clone()),
                    UIInput::container(UIPos::center(7, 4), 1.0, vec![vec![
                        UIInput::knob(
                            id_f_env_rel,
                            String::from("Length (ms)"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            id_env_slope,
                            String::from("Env Slope"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            id_env_rel,
                            String::from("Release (ms)"),
                            UIPos::center(4, 12).bottom()),
                    ]]),
                ],
                vec![
                    UIInput::graph_huge(
                        0,
                        String::from("Freq. Env"),
                        UIPos::center(5, 4).bottom(),
                        f_env_fun.clone()),
                    UIInput::container(UIPos::center(7, 4), 1.0, vec![vec![
                        UIInput::knob(
                            id_s_freq,
                            String::from("Start (Hz)"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            id_e_freq,
                            String::from("End (Hz)"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            id_f_slope,
                            String::from("Slope"),
                            UIPos::center(4, 12).bottom()),
                    ]]),
                ],
                vec![
                    UIInput::container_border(
                        UIPos::left(5, 4).bottom(),
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
                        UIPos::center(3, 4).middle()),
                    UIInput::btn_toggle(
                        id_n_e_freq,
                        String::from("E. from Note"),
                        UIPos::center(3, 4).middle()),
                ],
            ]);

    let mixer_params =
        UIInput::container_border(UIPos::center(12, 4), 1.0,
            vec![
                vec![
                    UIInput::knob(
                        id_gain,
                        String::from("Gain"),
                        UIPos::center(6, 12).bottom()),
                    UIInput::knob(
                        id_noise,
                        String::from("Noise"),
                        UIPos::center(6, 12).bottom()),
                ],
            ]);

    let dist_params =
        UIInput::Tabs(UITabData {
            pos: UIPos::center(12, 6),
            id: id_dist_on,
            labels: vec![
                String::from("Off"),
                String::from("Distortion"),
            ],
            childs: vec![
                vec![vec![
                    UIInput::label("Distortion off", 14.0, UIPos::center(6, 6).middle()),
                ]],
                vec![vec![
                    UIInput::container(UIPos::center(12, 12), 1.0,
                        vec![
                            vec![
                                UIInput::knob(
                                    id_d_start,
                                    String::from("Start"),
                                    UIPos::center(6, 12).middle()),
                                UIInput::knob(
                                    id_d_end,
                                    String::from("End"),
                                    UIPos::center(6, 12).middle()),
                            ],
                        ])
                ]],
            ],
        });

    gui.define_layout(vec![
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
                            String::from("Help"),
                        ],
                        childs: vec![
                            vec![vec![
                            UIInput::container(UIPos::center(12, 12), 1.0, vec![
                                vec![
                                    oscillator_params,
                                    UIInput::container(UIPos::center(4, 12), 1.0, vec![
                                        vec![ mixer_params ],
                                        vec![ dist_params ],
                                    ]),
                                ],
                            ])]],
                            vec![
                                vec![
                                    UIInput::Tabs(UITabData {
                                        pos: UIPos::center(12, 12),
                                        id: id_lic_tab,
                                        labels: vec![
                                            String::from("Input"),
                                            String::from("Copying"),
                                            String::from("Fonts"),
                                        ],
                                        childs: vec![
                                    vec![vec![UIInput::label_mono(
r#"Keyboard / Mouse Controls

    F1                  - Enter Help mode for elements
    Middle Mouse Button - Set Default value
    Right Mouse Button  - Enter value input mode
    Enter               - Accept entered value in value input mode
    Escape              - Exit help or value input mode
    Shift + Drag        - fine adjustment
"#,
                                        14.0,
                                        UIPos::left(12, 12).top())]],
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
            ],
        },
    ]);
}

impl KickmessEditor {
    pub(crate) fn new(host: HostCallback, params: Arc<KickmessVSTParams>) -> Self {
        Self {
            controller: Arc::new(KickmessEditorController {
                host,
                params,
                is_open: std::sync::atomic::AtomicBool::new(true),
                close_request: std::sync::atomic::AtomicBool::new(false),
            }),
        }
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
        crate::window::open_window(
            "Kickmess",
            WINDOW_WIDTH, WINDOW_HEIGHT,
            Some(parent), self.controller.clone());

        true
    }

    fn is_open(&mut self) -> bool {
        self.controller.is_still_open()
    }

    fn idle(&mut self) {
    }

    fn close(&mut self) {
        self.controller.request_close();
    }
}
