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

    cl_hdl.tx.send(UICmd::DefineValues(vec![
        UIValueSpec::new_id(),
        UIValueSpec::new_min_max(5.0, 5000.0, 6, 1).steps(0.05, 0.001),
        UIValueSpec::new_id(),
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

    cl_hdl.tx.send(UICmd::Define(vec![
        UILayout::Container {
            label: String::from("Test"),
            xv: 1,
            yv: 1,
            wv: 10,
            hv: 10,
            rows: vec![
                vec![
                    UIInput::knob(      1, String::from("SFreq."), UIPos::right(3, 4)),
                    UIInput::knob_small(1, String::from("SFreq."), UIPos::right(2, 4)),
                    UIInput::knob_huge( 1, String::from("SFreq."), UIPos::right(3, 4)),
                    UIInput::knob_huge( 1, String::from("SFreq."), UIPos::right(4, 4)),
                ],
                vec![
                    UIInput::knob(      1, String::from("SFreq."), UIPos::center(3, 4)),
                    UIInput::knob_small(1, String::from("SFreq."), UIPos::center(2, 4)),
                    UIInput::knob_huge( 1, String::from("SFreq."), UIPos::center(3, 4)),
                    UIInput::knob_huge( 1, String::from("SFreq."), UIPos::center(4, 4)),
                ],
                vec![
                    UIInput::knob(      1, String::from("SFreq."), UIPos::left(3, 4)),
                    UIInput::knob_small(1, String::from("SFreq."), UIPos::left(2, 4)),
                    UIInput::knob_huge( 1, String::from("SFreq."), UIPos::left(3, 4)),
                    UIInput::btn_2state(1,
                        String::from("Dist"),
                        String::from("On"),
                        String::from("Off"),
                        UIPos::left(4, 4)),
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
            println!("MSG FROM UI: {:?}", msg);
        }

        hdl.update_ui();
    }
}
