use crate::ui::painting::*;
use crate::ui::protocol::UIKnobData;
use crate::ui::protocol::UIBtnData;
use crate::ui::protocol::UIGraphData;

pub trait UIElement {
    fn size(&self) -> (f64, f64);
    fn draw_value(&self, p: &dyn Painter, x: f64, y: f64, highlight: HLStyle, data: &dyn UIElementData, value: f64, val_s: &str);
    fn draw_bg(&self, p: &dyn Painter);
    fn define_active_zones(&self, x: f64, y: f64, data: &dyn UIElementData, f: &mut dyn FnMut(ActiveZone));
}

pub trait UIElementData {
    fn as_knob_data(&self) -> Option<&UIKnobData> { None }
    fn as_graph_data(&self) -> Option<&UIGraphData> { None }
    fn as_btn_data(&self) -> Option<&UIBtnData> { None }
    fn value_id(&self) -> usize;
}
