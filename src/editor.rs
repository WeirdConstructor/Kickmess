// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use crate::KickmessVSTParams;
use vst::editor::{Editor, KeyCode};
use std::sync::Arc;
use std::rc::Rc;
use vst::plugin::{HostCallback};
use vst::host::Host;
use ringbuf::RingBuffer;
use keyboard_types::KeyboardEvent;

use crate::ui::protocol::*;
use crate::ui::constants::*;
use crate::ui;

const MAX_KEY_EVENTS_PER_FRAME : usize = 128;

pub const WINDOW_WIDTH:  i32 = 1000;
pub const WINDOW_HEIGHT: i32 =  700;

enum VSTKeyEvent {
    Pressed(KeyboardEvent),
    Released(KeyboardEvent),
}

pub(crate) struct KickmessEditorController {
    host:           HostCallback,
    params:         Arc<KickmessVSTParams>,
    is_open:        std::sync::atomic::AtomicBool,
    close_request:  std::sync::atomic::AtomicBool,
    key_events_tx:  ringbuf::Producer<VSTKeyEvent>,
    key_events_rx:  ringbuf::Consumer<VSTKeyEvent>,
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

        ui.set_version(crate::VERSION);

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

        while let Some(vst_ev) = self.key_events_rx.pop() {
            match vst_ev {
                VSTKeyEvent::Pressed(kev)  => ui.key_pressed(kev),
                VSTKeyEvent::Released(kev) => ui.key_released(kev),
            }
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

    fn fetch_logs(&self) -> Option<String> {
        self.params.gui_log.collect()
    }
}

pub fn define_gui(ps: &crate::ParamSet, gui: &mut dyn ui::protocol::UI) {
    let mut values = vec![];
    values.resize(45, UIValueSpec::new_id());

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
    let id_f1_cutoff = 14;
    let id_f1_res    = 15;
    let id_f1_type   = 16;
    let id_f1_drive  = 17;
    let id_f1_on     = 18;
    let id_o1_gain   = 19;
    let id_o1_wave   = 20;
    let id_o1_pw     = 21;
    let id_o1_unison = 22;
    let id_o1_detune = 23;

    let id_of1_freq  = 24;
    let id_of1_self  = 25;
    let id_of1_o2    = 26;
    let id_of2_o1    = 27;
    let id_of2_freq  = 28;
    let id_of2_self  = 29;
    let id_of2_gain  = 30;

    let id_main_gain = 31;

    let id_main_tab  = 40;
    let id_lic_tab   = 41;

    for i in 0..ps.param_count() {
        let help_text =
            if i < crate::param_model::help_texts.len() {
                crate::param_model::help_texts[i]
            } else { ("", "") };
        values[i] =
            ps.definition(i).unwrap()
              .to_ui_value_spec()
              .help(help_text.0, help_text.1);
    }

    let ht = crate::param_model::help_texts[id_n_s_freq];
    values[id_n_s_freq] = UIValueSpec::new_toggle(&[ "Off", "On" ]).help(ht.0, ht.1);
    let ht = crate::param_model::help_texts[id_n_e_freq];
    values[id_n_e_freq] = UIValueSpec::new_toggle(&[ "Off", "On" ]).help(ht.0, ht.1);
    let ht = crate::param_model::help_texts[id_dist_on];
    values[id_dist_on]  = UIValueSpec::new_toggle(&[ "Off", "On" ]).help(ht.0, ht.1);
    let ht = crate::param_model::help_texts[id_f1_on];
    values[id_f1_on]    = UIValueSpec::new_toggle(&[ "Off", "On" ]).help(ht.0, ht.1);

    values[id_f1_type]  = UIValueSpec::new_toggle(&[ "LP", "HP", "BP" ]).help(ht.0, ht.1);


    values[id_d_start]  .set_active_when_gt05(id_dist_on);
    values[id_d_end]    .set_active_when_gt05(id_dist_on);

    values[id_f1_cutoff].set_active_when_gt05(id_f1_on);
    values[id_f1_res]   .set_active_when_gt05(id_f1_on);
    values[id_f1_type]  .set_active_when_gt05(id_f1_on);
    values[id_f1_drive] .set_active_when_gt05(id_f1_on);

    values[id_o1_wave]  .set_active_when_gt0(id_o1_gain);
    values[id_o1_pw]    .set_active_when_gt0(id_o1_gain);
    values[id_o1_unison].set_active_when_gt0(id_o1_gain);
    values[id_o1_detune].set_active_when_gt0(id_o1_gain);

    gui.define_value_spec(values);

    let id_s_freq_f     = id_s_freq;
    let id_ae_f_env_rel = id_f_env_rel;
    let id_ae_f_slope   = id_f_slope;
    let id_ae_s_freq    = id_s_freq;
    let id_ae_e_freq    = id_e_freq;
    let f_env_fun =
        Arc::new(move |_id: usize, src: &mut dyn UIValueSource, out: &mut Vec<(f64, f64)>| {
            let min_x = 0.2;
            let max_x =
                min_x + (1.0 - min_x) * src.param_value(id_ae_f_env_rel).sqrt();
            let slope = src.param_value(id_ae_f_slope).max(0.01);

            let (sign, y_offs) =
                if (src.param_value_denorm(id_ae_s_freq)
                    - src.param_value_denorm(id_ae_e_freq))
                   < 0.0 {

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
        Arc::new(move |_id: usize,
                       src: &mut dyn UIValueSource,
                       out: &mut Vec<(f64, f64)>| {

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
        Arc::new(move |_id: usize,
                       src: &mut dyn UIValueSource,
                       out: &mut Vec<(f64, f64)>| {

            let samples = 80;

            for x in 0..(samples + 1) {
                let x = x as f64 / (samples as f64);
                out.push((
                    x,
                    (((x + 0.25 * src.param_value(id_ph_click))
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
        UIInput::container_border(UIPos::center(6, 12), 1.0, "Main Oscillator",
            vec![
                vec![
                    UIInput::graph_huge(
                        0,
                        String::from("Amp Env"),
                        UIPos::center(4, 2).bottom(),
                        amp_env_fun.clone()),
                    UIInput::container(UIPos::center(8, 2), 1.0, "Amp", vec![vec![
                        UIInput::knob(
                            id_f_env_rel,
                            String::from("Length (ms)"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            id_env_slope,
                            String::from("Amp Slope"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            id_env_rel,
                            String::from("Rel (ms)"),
                            UIPos::center(4, 12).bottom()),
                    ]]),
                ],
                vec![
                    UIInput::graph_huge(
                        0,
                        String::from("Freq. Env"),
                        UIPos::center(4, 2).bottom(),
                        f_env_fun.clone()),
                    UIInput::container(UIPos::center(8, 2), 1.0, "Pitch", vec![vec![
                        UIInput::knob(
                            id_s_freq,
                            String::from("Start Hz"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            id_e_freq,
                            String::from("End Hz"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            id_f_slope,
                            String::from("Freq Slope"),
                            UIPos::center(4, 12).bottom()),
                    ]]),
                ],
                vec![
                    UIInput::container_border(
                        UIPos::center(4, 2).middle(),
                        0.9,
                        "",
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
                    UIInput::container(UIPos::center(8, 2), 1.0, "", vec![vec![
                        UIInput::btn_toggle(
                            id_n_s_freq,
                            String::from("Note>St. F"),
                            UIPos::center(4, 12).middle()),
                        UIInput::btn_toggle(
                            id_n_e_freq,
                            String::from("Note>End F"),
                            UIPos::center(4, 12).middle()),
                    ]]),
                ],
                vec![
                    UIInput::container_border(UIPos::center(12, 3), 1.01, "Filter 1", vec![vec![
                        UIInput::btn_toggle_small(
                            id_f1_on,
                            String::from("Filter 1"),
                            UIPos::center(2, 12).middle()),
                        UIInput::container(UIPos::center(10, 12), 1.0, "", vec![vec![
                            UIInput::knob(
                                id_f1_cutoff,
                                String::from("F1 Cut"),
                                UIPos::center(3, 12).middle()),
                            UIInput::knob(
                                id_f1_res,
                                String::from("F1 Res"),
                                UIPos::center(3, 12).middle()),
                            UIInput::btn_toggle(
                                id_f1_type,
                                String::from("F1 Type"),
                                UIPos::left(3, 12).top()),
                            UIInput::knob(
                                id_f1_drive,
                                String::from("F1 Drive"),
                                UIPos::center(3, 12).middle()),
                        ]])
                    ]]),
                ],
                vec![
                    UIInput::container_border(UIPos::center(12, 3), 1.0, "Oscillator 1", vec![vec![
                        UIInput::knob(
                            id_o1_gain,
                            String::from("Osc1 Gain"),
                            UIPos::center(2, 12).middle()),
                        UIInput::knob(
                            id_o1_wave,
                            String::from("Osc1 Wave"),
                            UIPos::center(2, 12).middle()),
                        UIInput::knob(
                            id_o1_pw,
                            String::from("Osc1 PW"),
                            UIPos::center(2, 12).middle()),
                        UIInput::knob(
                            id_o1_unison,
                            String::from("Osc1 Uni."),
                            UIPos::center(2, 12).middle()),
                        UIInput::knob(
                            id_o1_detune,
                            String::from("Osc1 Det."),
                            UIPos::center(2, 12).middle()),
                    ]]),
                ]
            ]);


    let fm_params =
        UIInput::container_border(UIPos::center(4, 12), 1.0, "FM Oscillator",
            vec![
                vec![
                    UIInput::knob(
                        id_of1_freq,
                        String::from("Op1 Hz"),
                        UIPos::center(4, 2).middle()),
                    UIInput::knob(
                        id_of2_freq,
                        String::from("Op2 Hz"),
                        UIPos::center(4, 2).middle()),
                    UIInput::knob(
                        id_of2_gain,
                        String::from("Gain"),
                        UIPos::center(4, 2).middle()),
                ],
                vec![
                    UIInput::knob(
                        id_of1_self,
                        String::from("Op1<o Hz"),
                        UIPos::center(3, 2).middle()),
                    UIInput::knob(
                        id_of2_self,
                        String::from("Op2<o Hz"),
                        UIPos::center(3, 2).middle()),
                    UIInput::knob(
                        id_of1_o2,
                        String::from("Op1>2 Hz"),
                        UIPos::center(3, 2).middle()),
                    UIInput::knob(
                        id_of2_o1,
                        String::from("Op2>1 Hz"),
                        UIPos::center(3, 2).middle()),
                ],
            ]);

    let mixer_params =
        UIInput::container_border(UIPos::center(12, 4), 1.0, "Mixer",
            vec![
                vec![
                    UIInput::knob(
                        id_gain,
                        String::from("MOsc Gain"),
                        UIPos::center(6, 6).middle()),
                    UIInput::knob(
                        id_noise,
                        String::from("Tone/Noise"),
                        UIPos::center(6, 6).middle()),
                ],
                vec![
                    UIInput::knob(
                        id_main_gain,
                        String::from("Main Gain"),
                        UIPos::center(6, 6).middle()),
                ],
            ]);

    let dist_params =
        UIInput::container_border(UIPos::center(12, 6), 1.0, "Distortion",
            vec![vec![
                UIInput::container(UIPos::center(12, 12), 1.0, "",
                    vec![
                        vec![
                            UIInput::btn_toggle(
                                id_dist_on,
                                String::from("Distortion"),
                                UIPos::left(12, 4).top()),
                        ],
                        vec![
                            UIInput::knob(
                                id_d_start,
                                String::from("Start Amt"),
                                UIPos::center(6, 8).middle()),
                            UIInput::knob(
                                id_d_end,
                                String::from("End Amt"),
                                UIPos::center(6, 8).middle()),
                        ],
                    ])
            ]]);

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
                            UIInput::container(UIPos::center(12, 12), 1.0, "", vec![
                                vec![
                                    oscillator_params,
                                    UIInput::container(UIPos::center(2, 12), 1.0, "", vec![
                                        vec![ mixer_params ],
                                        vec![ dist_params ],
                                    ]),
                                    fm_params,
                                ],
                            ])]],
                            vec![
                                vec![
                                    UIInput::Tabs(UITabData {
                                        pos: UIPos::center(12, 12),
                                        id: id_lic_tab,
                                        labels: vec![
                                            String::from("Usage"),
                                            String::from("Copying"),
                                            String::from("Fonts"),
                                        ],
                                        childs: vec![
                                    vec![vec![UIInput::label_mono(
r#"
About the knobs and adjustment areas:

    Coarse adjustment: Center of the knob (value label) dragging.
    Fine adjustment:   Label/Name of the knob dragging.

Mouse controls:

    Middle Mouse Button - Set Default value
    Right Mouse Button  - Enter value input mode
    Mouse Wheel Up/Down - Adjust knob value according to coarse/fine area

Keyboard controls:

    F1                  - Enter Help mode for elements.
                          The input elements (eg. Knobs) with extra
                          help text are highlighted in the UI.
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
        let buf = RingBuffer::<T>::new(MAX_KEY_EVENTS_PER_FRAME);
        let (prod, cons) = buf.split();

        Self {
            controller: Arc::new(KickmessEditorController {
                host,
                params,
                is_open: std::sync::atomic::AtomicBool::new(true),
                close_request: std::sync::atomic::AtomicBool::new(false),
                key_events_tx: prod,
                key_events_rx: cons,
            }),
        }
    }
}

//fn keycode_to_keyevent(is_down: bool, kc: KeyCode) -> KeyboardEvent {
//    let mut modifiers : u32 = 0;
//
//    if kc.modifiers & vst::api::ModifierKey::SHIFT {
//        modifiers |= keyboard_types::Modifier::SHIFT;
//    }
//    if kc.modifiers & vst::api::ModifierKey::ALT {
//        modifiers |= keyboard_types::Modifier::ALT;
//    }
//    if kc.modifiers & vst::api::ModifierKey::COMMAND {
//        modifiers |= keyboard_types::Modifier::COMMAND;
//    }
//    if kc.modifiers & vst::api::ModifierKey::CONTROL {
//        modifiers |= keyboard_types::Modifier::CONTROL;
//    }
//
//    let mut kev = KeyboardEvent {
//        state:          if is_down { keyboard_types::KeyState::Down }
//                        else       { keyboard_types::KeyState::Up },
//        location:   keyboard_types::Location::Standard,
//        modifiers,
//        repeat:         false,
//        is_composing:   false,
//    };
//    match kc.key {
//        vst::editor::Key::
//    }
//}

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

    fn key_up(&mut self, keyc: KeyCode) -> bool {
        println!("KEY UP {:?}", keyc);
        false
    }

    fn key_down(&mut self, keyc: KeyCode) -> bool {
        println!("KEY DOWN {:?}", keyc);
        false
    }
}
