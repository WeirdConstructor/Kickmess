// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use kickmessvst;
use kickmessvst::ui;
use kickmessvst::ui::protocol::*;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use kickmessvst::proc::Param;

fn main() {
    let (cl_hdl, p_hdl) = ui::protocol::UIClientHandle::create();

    let runner = kickmessvst::window::open_window(None, p_hdl);

    let graph_fun = Arc::new(|id: usize, src: &mut dyn UIGraphValueSource, out: &mut Vec<(f64, f64)>| {
        let samples = 40;
        for x in 0..(samples + 1) {
            let x = x as f64 / (samples as f64);
            out.push((
                x,
                ((x
                 * (4.0 * src.param_value(11) + 1.0)
                 * 2.0 * std::f64::consts::PI)
                .sin() + 1.0) / 2.0));
        }
    });

    cl_hdl.tx.send(UICmd::DefineValues(vec![
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

    cl_hdl.tx.send(UICmd::Define(vec![
        UILayout::Container {
            label: String::from("Test GUI"),
            xv: 1, yv: 1, wv: 7, hv: 10,
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
        UILayout::Container {
            label: String::from("Graph Test"),
            xv: 8, yv: 1, wv: 3, hv: 10,
            rows: vec![
                vec![
                    UIInput::graph(      1, String::from("Wavey"),  UIPos::center(12, 3), graph_fun.clone()),
                ],
                vec![
                    UIInput::graph_huge(      1, String::from("Wavey (h)"),  UIPos::center(12, 3), graph_fun.clone()),
                ],
                vec![
                    UIInput::container_border(UIPos::center(12, 5), vec![
                        vec![
                            UIInput::graph_small(1, String::from("Wavey (s)"),  UIPos::left(12, 5), graph_fun),
                        ],
                        vec![
                            UIInput::knob_small(11, String::from("w"),          UIPos::left(12, 7)),
                        ]
                    ])
                ],
                vec![
                ],
            ],
        },
    ])).expect("mpsc ok");

     // TODO: Send VALUES!

//    std::thread::spawn(move || {
//        while let Ok(msg) = cl_hdl.rx.recv_timeout(
//            std::time::Duration::from_millis(1000)) {
//            println!("MSG FROM UI: {:?}", msg);
//        }
//    });

    std::thread::spawn(move || {
        let mut closed = false;
        while !closed {
            while let Ok(msg) = cl_hdl.rx.recv() {
//                println!("MSG FROM UI: {:?}", msg);
                match msg {
                    UIMsg::WindowClosed => { closed = true; break; },
                    _ => {},
                }
            }
        }

//        while let Ok(msg) = cl_hdl.rx.recv_timeout(
//            std::time::Duration::from_millis(1000)) {
//            println!("MSG FROM UI: {:?}", msg);
//        }
    });

    runner.unwrap().app_run_blocking();
}
