use kickmessvst;
use kickmessvst::ui;
use kickmessvst::ui::protocol::*;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;

use kickmessvst::proc::Param;

fn main() {
    let (cl_hdl, p_hdl) = ui::protocol::UIClientHandle::create();

    let runner = kickmessvst::baseview_femtovg::open_window(None, p_hdl);

    cl_hdl.tx.send(UICmd::DefineValues(vec![
        UIValueSpec::new_id(),
        UIValueSpec::new_min_max(5.0, 9999.0, 6, 1).steps(0.01, 0.001),
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
                    UIInput::knob(      1, String::from("SFreq."), UIPos::center(2, 6)),
                    UIInput::knob_small(1, String::from("SFreq."), UIPos::center(2, 6)),
                    UIInput::knob_huge( 1, String::from("SFreq."), UIPos::center(2, 6)),
                ],
            ],
        },
    ])).expect("mpsc ok");

     // TODO: Send VALUES!

    std::thread::spawn(move || {
        while let Ok(msg) = cl_hdl.rx.recv_timeout(
            std::time::Duration::from_millis(1000)) {
            println!("MSG FROM UI: {:?}", msg);
        }
    });

    runner.unwrap().app_run_blocking();
}
