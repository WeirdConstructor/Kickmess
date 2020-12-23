//pub fn draw_left_text(cr: &cairo::Context, x: f64, y: f64, h: f64, txt: &str) {
//    let  ext = cr.text_extents(txt);
//    let fext = cr.font_extents();
//    cr.move_to(
//        x,
//        y + fext.height
//          + ((h - fext.height) / 2.0).abs().round()
//          - fext.descent);
//    cr.show_text(txt);
//}
//
//pub fn draw_centered_text(cr: &cairo::Context, x: f64, y: f64, w: f64, h: f64, txt: &str) {
//    let  ext = cr.text_extents(txt);
//    let fext = cr.font_extents();
//    cr.move_to(
//        x + ((w - ext.width) / 2.0).abs().round(),
//        y + fext.height
//          + ((h - fext.height) / 2.0).abs().round()
//          - fext.descent);
//    cr.show_text(txt);
//}
