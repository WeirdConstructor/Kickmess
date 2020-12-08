use crate::ui::painting::*;
use crate::ui::protocol::UIKnobData;
use crate::ui::protocol::UIBtnData;

pub trait UIElement {
    fn size(&self) -> (f64, f64);
    fn draw_value(&self, cr: &cairo::Context, x: f64, y: f64, hover_style: bool, data: &dyn UIElementData, value: f64, val_s: &str);
    fn draw_bg(&self, cr: &cairo::Context);
    fn define_active_zones(&self, x: f64, y: f64, f: &mut dyn FnMut(ActiveZone));
}

pub trait UIElementData {
    fn as_knob_data(&self) -> Option<&UIKnobData> { None }
    fn as_btn_data(&self) -> Option<&UIBtnData> { None }
    fn value_id(&self) -> usize;
}
