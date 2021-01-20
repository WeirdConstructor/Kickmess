// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use kickmessvst;
use kickmessvst::ui::protocol::*;
use std::sync::Arc;


struct TestController {
}

impl UIController for TestController {
    fn init(&self, ui: &mut dyn UI) {
        use kickmessvst::MonoProcessor;
//        self.is_open.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut ps        = kickmessvst::ParamSet::new();
        let mut public_ps = kickmessvst::ParamSet::new();
        kickmessvst::OpKickmess::init_params(&mut ps, &mut public_ps);
        kickmessvst::editor::define_gui(&ps, ui);
        ui.set_version(kickmessvst::VERSION);
    }

    fn window_closed(&self, _ui: &mut dyn UI) {
//        self.is_open.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    fn fetch_logs(&self) -> Option<String> {
        None
    }
}

fn main() {
    let ctrl = Arc::new(TestController { });

//    std::thread::spawn(move || {
//        let mut closed = false;
//        while !closed {
//            while let Ok(msg) = cl_hdl.rx.recv() {
//                match msg {
//                    UIMsg::WindowClosed => { closed = true; break; },
//                    _ => {},
//                }
//            }
//        }
//    });

    kickmessvst::window::open_window(
        "Kickmess Test GUI",
        kickmessvst::editor::WINDOW_WIDTH,
        kickmessvst::editor::WINDOW_HEIGHT,
        None, ctrl);


//    let graph_fun = Arc::new(|_id: usize, src: &mut dyn UIGraphValueSource, out: &mut Vec<(f64, f64)>| {
//        let samples = 40;
//        for x in 0..(samples + 1) {
//            let x = x as f64 / (samples as f64);
//            out.push((
//                x,
//                ((x
//                 * (4.0 * src.param_value(11) + 1.0)
//                 * 2.0 * std::f64::consts::PI)
//                .sin() + 1.0) / 2.0));
//        }
//    });
//
//    cl_hdl.tx.send(UICmd::DefineValues(vec![
//        UIValueSpec::new_id(),
//        UIValueSpec::new_min_max_exp(5.0, 3000.0, 6, 1).steps(0.04, 0.01).help("S Freq", "fie fwof ewiof ew\nfewfwiuofewoi fewoi fewoif \nfiewfoiew foiew jfewoij \nfwefiwfh weifuhi "),
//        UIValueSpec::new_min_max_exp(5.0, 2000.0, 6, 1).steps(0.04, 0.01).help("E Freq", "END fwof ewiof ew\nfewfwiuofewoi ENDoi fewoif \nfiewfoiew ENDew jfewoij \nfwefiwfh ENDfuhi "),
//        UIValueSpec::new_min_max_exp(5.0, 5000.0, 6, 1).steps(0.04, 0.01),
//        UIValueSpec::new_min_max(0.0, 100.0, 5, 1).steps(0.04, 0.01),
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
//    ])).expect("mpsc ok");
//
//    let tabs_i =
//        UIInput::Tabs(UITabData {
//            pos: UIPos::center(12, 12),
//            id: 10,
//            labels: vec![String::from("XXX"), String::from("YYY"), String::from("ZZZ")],
//            childs: vec![
//                vec![
//                    vec![
//                        UIInput::label("TAB 1", 14.0, UIPos::center(12, 6).middle()),
//                    ],
//                ],
//                vec![
//                    vec![
//                        UIInput::label("TAB 2", 14.0, UIPos::center(12, 6).middle()),
//                    ],
//                ],
//                vec![
//                    vec![
//                        UIInput::label("TAB 3", 14.0, UIPos::center(12, 6).middle()),
//                    ],
//                ],
//            ],
//        });
//
//
//    let tabs =
//        UIInput::Tabs(UITabData {
//            pos: UIPos::center(12, 12),
//            id: 11,
//            labels: vec![
//                String::from("ABCDEF"),
//                String::from("OFOFEOFE"),
//                String::from("FOFOO")
//            ],
//            childs: vec![
//                vec![
//                    vec![
//                        UIInput::label("TAB 1", 14.0, UIPos::center(12, 6).middle()),
//                    ],
//                    vec![
//                        UIInput::label("TAB 1", 14.0, UIPos::center(12, 6).middle()),
//                    ],
//                ],
//                vec![
//                    vec![
//                        UIInput::label("TAB 2", 14.0, UIPos::center(12, 6).middle()),
//                    ],
//                    vec![
//                        UIInput::label("TAB 2", 14.0, UIPos::center(12, 6).middle()),
//                    ],
//                ],
//                vec![
//                    vec![
//                        tabs_i
//                    ],
//                ],
//            ],
//        });
//
//    cl_hdl.tx.send(UICmd::Define(vec![
//        UILayout::Container {
//            label: String::from("Test GUI"),
//            xv: 1, yv: 1, wv: 7, hv: 8,
//            rows: vec![
//                vec![
//                    UIInput::container_border(UIPos::center(12, 4), vec![ vec![
//                            UIInput::knob(      1, String::from("Start (Hz)"),  UIPos::right(3, 12)),
//                            UIInput::knob_small(2, String::from("End (Hz)"),    UIPos::right(2, 12)),
//                            UIInput::knob_huge( 3, String::from("Length (ms)"), UIPos::right(3, 12)),
//                            UIInput::btn_mod_target(9, String::from("Mod1"),    UIPos::right(4, 12)),
//                    ], ]),
//                ],
//                vec![
//                    UIInput::container_border(UIPos::center(12, 4), vec![ vec![
//                        UIInput::knob(      4, String::from("Dist S."), UIPos::center(3, 12)),
//                        UIInput::knob_small(5, String::from("Dist E."), UIPos::center(2, 12)),
//                        UIInput::knob_huge( 1, String::from("SFreq."),  UIPos::center(3, 12)),
//                        UIInput::btn_toggle(10, String::from("Mod2"),   UIPos::center(4, 12)),
//                    ], ]),
//                ],
//                vec![
//                    UIInput::container_border(UIPos::center(12, 4), vec![ vec![
//                        UIInput::knob(      1, String::from("SFreq."),   UIPos::left(3, 12).bottom()),
//                        UIInput::knob_small(1, String::from("SFreq."),   UIPos::left(2, 12).bottom()),
//                        UIInput::knob_huge( 1, String::from("SFreq."),   UIPos::left(3, 12).bottom()),
//                        UIInput::btn_drag_value(7, String::from("Mod3"), UIPos::left(4, 12).bottom()),
//                    ], ]),
//                ],
//            ],
//        },
//        UILayout::Container {
//            label: String::from("Graph Test"),
//            xv: 8, yv: 1, wv: 3, hv: 8,
//            rows: vec![
//                vec![
//                    UIInput::graph(      1, String::from("Wavey"),  UIPos::center(12, 3), graph_fun.clone()),
//                ],
//                vec![
//                    UIInput::graph_huge(      1, String::from("Wavey (h)"),  UIPos::center(12, 3), graph_fun.clone()),
//                ],
//                vec![
//                    UIInput::container_border(UIPos::center(12, 5), vec![
//                        vec![
//                            UIInput::graph_small(1, String::from("Wavey (s)"),  UIPos::left(6, 5), graph_fun),
//                            UIInput::lines("Text ää\nfeof\nfeowfwe", 14.0, UIPos::center(6, 5).middle()),
//                        ],
//                        vec![
//                            UIInput::knob_small(11, String::from("w"),          UIPos::left(6, 7)),
//                            UIInput::lines_border_mono("- Text ää\nfeof\nfeowfwe", 13.0, UIPos::center(6, 4).middle()),
//                        ]
//                    ])
//                ],
//                vec![
//                ],
//            ],
//        },
//        UILayout::Container {
//            label: String::from(""),
//            xv: 1, yv: 9, wv: 10, hv: 3,
//            rows: vec![ vec![ tabs ], ],
//        }
//    ])).expect("mpsc ok");
//
     // TODO: Send VALUES!

//    std::thread::spawn(move || {
//        while let Ok(msg) = cl_hdl.rx.recv_timeout(
//            std::time::Duration::from_millis(1000)) {
//            println!("MSG FROM UI: {:?}", msg);
//        }
//    });

}
