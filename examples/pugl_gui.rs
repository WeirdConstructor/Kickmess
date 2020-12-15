use kickmessvst;
use kickmessvst::ui;
use kickmessvst::ui::protocol::*;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use kickmessvst::proc::Param;
use pugl_sys::*;

/*

Modulation Framework for Synth:

    - 2x DAHDSR Envelope
    - 2x LFO                (Speed)
    - 1 Filter              (Cutoff, Q)
    - 1 VCA                 (Gain)

    Env1   o            o Filter Cutoff
    Env2   o            o Filter Q
    LFO 1  o            o Osc Gain
    LFO 2  o            o LFO 1 Speed
                        o LFO 2 Speed

    Nf Function routes:

    [ F Cut / F Q / VCA / L1 Spd / L2 Spd ]
    /--------------\    /""\
    |  Graph plot  |    |  |
    |              |    \__/
    \--------------/ [ Amount ]
    [ x / 1 - x / 1 - x^y / x^y ]
    [Off / L1   / L2  /  E1  / E2     ]

*/

fn main() {
    let (cl_hdl, p_hdl) = ui::protocol::UIClientHandle::create();

    let mut view = kickmessvst::pugl::open_window(None, Some(p_hdl));

//        ps.add(ParamDefinition::from(Param::Freq1,      5.0,   3000.0, 150.0, "Start Freq."));
//        ps.add(ParamDefinition::from(Param::Freq2,      5.0,   2000.0,  40.0, "End Freq."));
//        ps.add(ParamDefinition::from(Param::Decay1,     5.0,   5000.0, 440.0, "Length"));
//        ps.add(ParamDefinition::from(Param::Dist1,      0.0,   100.0,    0.8, "Dist. Start"));
//        ps.add(ParamDefinition::from(Param::Dist2,      0.0,   100.0,    0.8, "Dist. End"));
//        ps.add(ParamDefinition::from(Param::Gain1,      0.1,   5.0,      1.0, "Dist. Gain"));
//        ps.add(ParamDefinition::from(Param::Env1,       0.01,  1.0,    0.163, "Env. slope"));
//        ps.add(ParamDefinition::from(Param::Release1,   0.001, 1.0,     0.06, "Freq. slope"));
//        ps.add(ParamDefinition::from(Param::Noise1,     0.0,   1.0,      0.0, "Noise"));
//        ps.add(ParamDefinition::from(Param::S1,         0.0,   1.0,      1.0, "Start from note"));
//        ps.add(ParamDefinition::from(Param::Release2,   1.0,1000.0,      5.0, "Env Release"));
//        ps.add(ParamDefinition::from(Param::Phase1,     0.0,   1.0,      0.0, "Click"));

    cl_hdl.tx.send(UICmd::DefineValues(vec![
        UIValueSpec::new_id(),
        UIValueSpec::new_min_max(5.0, 3000.0, 6, 1).steps(0.05, 0.001),
        UIValueSpec::new_min_max(5.0, 2000.0, 6, 1).steps(0.05, 0.001),
        UIValueSpec::new_min_max(5.0, 5000.0, 6, 1).steps(0.05, 0.001),
        UIValueSpec::new_min_max(0.0, 100.0, 5, 1).steps(0.05, 0.001),
        UIValueSpec::new_min_max(0.0, 100.0, 5, 1).steps(0.05, 0.001),
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
                    UIInput::knob(      1, String::from("Start (Hz)"),  UIPos::right(3, 4)),
                    UIInput::knob_small(2, String::from("End (Hz)"),    UIPos::right(2, 4)),
                    UIInput::knob_huge( 3, String::from("Length (ms)"), UIPos::right(3, 4)),
                    UIInput::btn_mod_target(9, String::from("Mod1"), UIPos::right(4, 4)),
                ],
                vec![
                    UIInput::knob(      4, String::from("Dist S."), UIPos::center(3, 4)),
                    UIInput::knob_small(5, String::from("Dist E."), UIPos::center(2, 4)),
                    UIInput::knob_huge( 1, String::from("SFreq."), UIPos::center(3, 4)),
                    UIInput::btn_toggle(10, String::from("Mod2"), UIPos::center(4, 4)),
                ],
                vec![
                    UIInput::knob(      1, String::from("SFreq."), UIPos::left(3, 4)),
                    UIInput::knob_small(1, String::from("SFreq."), UIPos::left(2, 4)),
                    UIInput::knob_huge( 1, String::from("SFreq."), UIPos::left(3, 4)),
                    UIInput::btn_drag_value(7, String::from("Mod3"), UIPos::left(4, 4)),
                ],
            ],
        },
        UILayout::Container {
            label: String::from("Graph Test"),
            xv: 8, yv: 1, wv: 3, hv: 10,
            rows: vec![
                vec![
                    UIInput::graph(      1, String::from("Wavey"),  UIPos::center(12, 3)),
                ],
                vec![
                    UIInput::graph_huge(      1, String::from("Wavey (h)"),  UIPos::center(12, 3)),
                ],
                vec![
                    UIInput::graph_small(      1, String::from("Wavey (s)"),  UIPos::center(12, 3)),
                ],
                vec![
                    UIInput::knob(      11, String::from("w"),      UIPos::center(12, 3)),
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

    let mut hdl = view.as_mut().handle();
    let mut closed = false;
    while !closed {
        hdl.update(0.01);

        while let Ok(msg) = cl_hdl.rx.try_recv() {
            match msg {
                UIMsg::WindowClosed => { closed = true; break; },
                _ => {},
            }
//            println!("MSG FROM UI: {:?}", msg);
        }

        hdl.update_ui();
    }
}
