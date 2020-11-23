use kickmessvst;


fn main() {
    let handle = kickmessvst::baseview::open_window(None);

    handle.app_run_blocking();
}
