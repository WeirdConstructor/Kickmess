use crate::ui::painting::*;

pub trait UIElement {
    fn size(&self, line_width: f64) -> (f64, f64);
    fn draw_value(&self, cr: &cairo::Context, x: f64, y: f64, hover_style: bool, name: &str, value: f64, val_s: &str);
    fn draw_bg(&self, cr: &cairo::Context, x: f64, y: f64);
    fn define_active_zones(&self, x: f64, y: f64, f: &mut dyn FnMut(ActiveZone));
}
