use kickmess::baseview;


fn main() {
    let handle = baseview::open_window(None);

    handle.app_run_blocking();
}
