// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use crate::KickmessVSTParams;
use vst::editor::{Editor, KeyCode};
use std::sync::Arc;
use std::rc::Rc;
use vst::plugin::{HostCallback};
use vst::host::Host;
use crate::ringbuf_shared::RingBuf;
use keyboard_types::KeyboardEvent;

use crate::ui::protocol::*;
use crate::ui::constants::*;
use crate::ui;

const MAX_KEY_EVENTS_PER_FRAME : usize = 128;

pub const WINDOW_WIDTH:  i32 = 1000;
pub const WINDOW_HEIGHT: i32 =  700;

#[derive(Debug, Clone)]
enum VSTKeyEvent {
    Pressed(KeyboardEvent),
    Released(KeyboardEvent),
}

pub(crate) struct KickmessEditorController {
    host:           HostCallback,
    params:         Arc<KickmessVSTParams>,
    is_open:        std::sync::atomic::AtomicBool,
    close_request:  std::sync::atomic::AtomicBool,
    key_events:     RingBuf<VSTKeyEvent>,
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

        while let Some(vst_ev) = self.key_events.pop() {
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
    use crate::param_model::pid::{self};
    use crate::param_model::PARAM_COUNT;

    let mut values = vec![];
    values.resize(PARAM_COUNT + 2, UIValueSpec::new_id());

    let id_main_tab  = PARAM_COUNT;
    let id_lic_tab   = PARAM_COUNT + 1;

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

    let ht = crate::param_model::help_texts[pid::freq_note_start];
    values[pid::freq_note_start] = UIValueSpec::new_toggle(&[ "Off", "On" ]).help(ht.0, ht.1);
    let ht = crate::param_model::help_texts[pid::freq_note_end];
    values[pid::freq_note_end] = UIValueSpec::new_toggle(&[ "Off", "On" ]).help(ht.0, ht.1);
    let ht = crate::param_model::help_texts[pid::dist_on];
    values[pid::dist_on]  = UIValueSpec::new_toggle(&[ "Off", "On" ]).help(ht.0, ht.1);
    let ht = crate::param_model::help_texts[pid::f1_on];
    values[pid::f1_on]    = UIValueSpec::new_toggle(&[ "Off", "On" ]).help(ht.0, ht.1);

    let ht = crate::param_model::help_texts[pid::o2fm_mode];
    values[pid::o2fm_mode] = UIValueSpec::new_toggle(&[ "Env", "Fixed" ]).help(ht.0, ht.1);

    values[pid::f1_type]  = UIValueSpec::new_toggle(&[ "LP", "HP", "BP" ]).help(ht.0, ht.1);


    values[pid::dist_start] .set_active_when_gt05(pid::dist_on);
    values[pid::dist_end]   .set_active_when_gt05(pid::dist_on);

    values[pid::f1_cutoff]  .set_active_when_gt05(pid::f1_on);
    values[pid::f1_res]     .set_active_when_gt05(pid::f1_on);
    values[pid::f1_type]    .set_active_when_gt05(pid::f1_on);
    values[pid::f1_drive]   .set_active_when_gt05(pid::f1_on);

    values[pid::o1_waveform].set_active_when_gt0(pid::o1_gain);
    values[pid::o1_pw]      .set_active_when_gt0(pid::o1_gain);
    values[pid::o1_unison]  .set_active_when_gt0(pid::o1_gain);
    values[pid::o1_detune]  .set_active_when_gt0(pid::o1_gain);

    values[pid::o1fm_ratio] .set_active_when_gt0(pid::o2fm_gain);
    values[pid::o1fm_self]  .set_active_when_gt0(pid::o2fm_gain);
    values[pid::o1fm_o2_mod].set_active_when_gt0(pid::o2fm_gain);
    values[pid::o2fm_o1_mod].set_active_when_gt0(pid::o2fm_gain);
    values[pid::o2fm_self]  .set_active_when_gt0(pid::o2fm_gain);

    values[pid::o2fm_freq]  .set_active_when_gt05(pid::o2fm_mode);

    gui.define_value_spec(values);

    let id_s_freq_f     = pid::freq_start;
    let id_ae_f_env_rel = pid::f_env_release;
    let id_ae_f_slope   = pid::freq_slope;
    let id_ae_s_freq    = pid::freq_start;
    let id_ae_e_freq    = pid::freq_end;
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

    let id_s_freq_f     = pid::freq_start;
    let id_ae_f_env_rel = pid::f_env_release;
    let id_ae_env_slope = pid::env_slope;
    let id_ae_s_freq    = pid::freq_start;
    let id_ae_e_freq    = pid::freq_end;
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

    let phase_fun =
        Arc::new(move |_id: usize,
                       src: &mut dyn UIValueSource,
                       out: &mut Vec<(f64, f64)>| {

            let samples = 80;

            for x in 0..(samples + 1) {
                let x = x as f64 / (samples as f64);
                out.push((
                    x,
                    (((x + 0.25 * src.param_value(pid::phase_offs))
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
                            pid::f_env_release,
                            String::from("Length (ms)"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            pid::env_slope,
                            String::from("Amp Slope"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            pid::env_release,
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
                            pid::freq_start,
                            String::from("Start Hz"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            pid::freq_end,
                            String::from("End Hz"),
                            UIPos::center(4, 12).bottom()),
                        UIInput::knob(
                            pid::freq_slope,
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
                                    pid::phase_offs,
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
                            pid::freq_note_start,
                            String::from("Note>St. F"),
                            UIPos::center(4, 12).middle()),
                        UIInput::btn_toggle(
                            pid::freq_note_end,
                            String::from("Note>End F"),
                            UIPos::center(4, 12).middle()),
                    ]]),
                ],
                vec![
                    UIInput::container_border(UIPos::center(12, 3), 1.01, "Filter 1", vec![vec![
                        UIInput::btn_toggle_small(
                            pid::f1_on,
                            String::from("Filter 1"),
                            UIPos::center(2, 12).middle()),
                        UIInput::container(UIPos::center(10, 12), 1.0, "", vec![vec![
                            UIInput::knob(
                                pid::f1_cutoff,
                                String::from("F1 Cut"),
                                UIPos::center(3, 12).middle()),
                            UIInput::knob(
                                pid::f1_res,
                                String::from("F1 Res"),
                                UIPos::center(3, 12).middle()),
                            UIInput::btn_toggle(
                                pid::f1_type,
                                String::from("F1 Type"),
                                UIPos::left(3, 12).top()),
                            UIInput::knob(
                                pid::f1_drive,
                                String::from("F1 Drive"),
                                UIPos::center(3, 12).middle()),
                        ]])
                    ]]),
                ],
                vec![
                    UIInput::container_border(UIPos::center(12, 3), 1.0, "Oscillator 1", vec![vec![
                        UIInput::knob(
                            pid::o1_gain,
                            String::from("Osc1 Gain"),
                            UIPos::center(2, 12).middle()),
                        UIInput::knob(
                            pid::o1_waveform,
                            String::from("Osc1 Wave"),
                            UIPos::center(2, 12).middle()),
                        UIInput::knob(
                            pid::o1_pw,
                            String::from("Osc1 PW"),
                            UIPos::center(2, 12).middle()),
                        UIInput::knob(
                            pid::o1_unison,
                            String::from("Osc1 Uni."),
                            UIPos::center(2, 12).middle()),
                        UIInput::knob(
                            pid::o1_detune,
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
                        pid::o1fm_ratio,
                        String::from("Op1 Hz"),
                        UIPos::center(3, 2).middle()),
                    UIInput::knob(
                        pid::o2fm_freq,
                        String::from("Op2 Hz"),
                        UIPos::center(3, 2).middle()),
                    UIInput::btn_toggle(
                        pid::o2fm_mode,
                        String::from("Op2 Pitch"),
                        UIPos::center(3, 2).middle()),
                    UIInput::knob(
                        pid::o2fm_gain,
                        String::from("Gain"),
                        UIPos::center(3, 2).middle()),
                ],
                vec![
                    UIInput::knob(
                        pid::o1fm_self,
                        String::from("Op1<o Hz"),
                        UIPos::center(3, 2).middle()),
                    UIInput::knob(
                        pid::o2fm_self,
                        String::from("Op2<o Hz"),
                        UIPos::center(3, 2).middle()),
                    UIInput::knob(
                        pid::o1fm_o2_mod,
                        String::from("Op1>2 Hz"),
                        UIPos::center(3, 2).middle()),
                    UIInput::knob(
                        pid::o2fm_o1_mod,
                        String::from("Op2>1 Hz"),
                        UIPos::center(3, 2).middle()),
                ],
            ]);

    let mixer_params =
        UIInput::container_border(UIPos::center(12, 4), 1.0, "Mixer",
            vec![
                vec![
                    UIInput::knob(
                        pid::gain,
                        String::from("MOsc Gain"),
                        UIPos::center(6, 6).middle()),
                    UIInput::knob(
                        pid::noise,
                        String::from("Tone/Noise"),
                        UIPos::center(6, 6).middle()),
                ],
                vec![
                    UIInput::knob(
                        pid::main_gain,
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
                                pid::dist_on,
                                String::from("Distortion"),
                                UIPos::left(12, 4).top()),
                        ],
                        vec![
                            UIInput::knob(
                                pid::dist_start,
                                String::from("Start Amt"),
                                UIPos::center(6, 8).middle()),
                            UIInput::knob(
                                pid::dist_end,
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
        Self {
            controller: Arc::new(KickmessEditorController {
                host,
                params,
                is_open: std::sync::atomic::AtomicBool::new(true),
                close_request: std::sync::atomic::AtomicBool::new(false),
                key_events: RingBuf::new(MAX_KEY_EVENTS_PER_FRAME),
            }),
        }
    }
}

fn keycode_to_keyevent(is_down: bool, kc: KeyCode) -> KeyboardEvent {
    let mut modifiers : keyboard_types::Modifiers =
        keyboard_types::Modifiers::empty();

    let mut buf = [0; 8];
    let key =
        match kc.key {
            vst::editor::Key::None
                => keyboard_types::Key::Character(
                    kc.character.encode_utf8(&mut buf).to_string()),
            vst::editor::Key::Tab      => keyboard_types::Key::Tab,
            vst::editor::Key::Back     => keyboard_types::Key::Backspace,
            vst::editor::Key::Return   => keyboard_types::Key::Enter,
            vst::editor::Key::Enter    => keyboard_types::Key::Enter,
            vst::editor::Key::Escape   => keyboard_types::Key::Escape,
            vst::editor::Key::End      => keyboard_types::Key::End,
            vst::editor::Key::Home     => keyboard_types::Key::Home,
            vst::editor::Key::Left     => keyboard_types::Key::ArrowLeft,
            vst::editor::Key::Up       => keyboard_types::Key::ArrowUp,
            vst::editor::Key::Right    => keyboard_types::Key::ArrowRight,
            vst::editor::Key::Down     => keyboard_types::Key::ArrowDown,
            vst::editor::Key::PageUp   => keyboard_types::Key::PageUp,
            vst::editor::Key::PageDown => keyboard_types::Key::PageDown,
            vst::editor::Key::Insert   => keyboard_types::Key::Insert,
            vst::editor::Key::Delete   => keyboard_types::Key::Delete,
            vst::editor::Key::Help     => keyboard_types::Key::Help,
            vst::editor::Key::F1       => keyboard_types::Key::F1,
            vst::editor::Key::F2       => keyboard_types::Key::F2,
            vst::editor::Key::F3       => keyboard_types::Key::F3,
            vst::editor::Key::F4       => keyboard_types::Key::F4,
            vst::editor::Key::F5       => keyboard_types::Key::F5,
            vst::editor::Key::F6       => keyboard_types::Key::F6,
            vst::editor::Key::F7       => keyboard_types::Key::F7,
            vst::editor::Key::F8       => keyboard_types::Key::F8,
            vst::editor::Key::F9       => keyboard_types::Key::F9,
            vst::editor::Key::F10      => keyboard_types::Key::F10,
            vst::editor::Key::F11      => keyboard_types::Key::F11,
            vst::editor::Key::F12      => keyboard_types::Key::F12,
            vst::editor::Key::Shift    => keyboard_types::Key::Shift,
            vst::editor::Key::Control  => keyboard_types::Key::Control,
            vst::editor::Key::Alt      => keyboard_types::Key::Alt,
            _ => keyboard_types::Key::Unidentified
        };

    let mut kev = KeyboardEvent {
        key,
        state:          if is_down { keyboard_types::KeyState::Down }
                        else       { keyboard_types::KeyState::Up },
        code:           keyboard_types::Code::Unidentified,
        location:       keyboard_types::Location::Standard,
        modifiers,
        repeat:         false,
        is_composing:   false,
    };

    kev
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

    fn key_up(&mut self, keyc: KeyCode) -> bool {
        self.controller.key_events.push(
            VSTKeyEvent::Released(
                keycode_to_keyevent(false, keyc)));
        true
    }

    fn key_down(&mut self, keyc: KeyCode) -> bool {
        self.controller.key_events.push(
            VSTKeyEvent::Pressed(
                keycode_to_keyevent(true, keyc)));
        true
    }
}
