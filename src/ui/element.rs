// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use crate::ui::painting::*;
use crate::ui::protocol::UIKnobData;
use crate::ui::protocol::UIBtnData;
use crate::ui::protocol::UIGraphData;

//#[derive(Debug, Clone, Copy)]
//pub enum UIElementSize {
//    Normal,
//    Small,
//    Huge
//}

pub trait UIElement {
    fn size(&self) -> (f64, f64);
    fn draw_value(&self, p: &mut dyn Painter, x: f64, y: f64, highlight: HLStyle, data: &dyn UIElementData, value: f64, val_s: &str);
    fn draw_bg(&self, p: &mut dyn Painter, x: f64, y: f64);
    fn define_active_zones(&self, x: f64, y: f64, data: &dyn UIElementData, f: &mut dyn FnMut(ActiveZone));
}

pub trait UIElementData {
    fn as_knob_data(&self) -> Option<&UIKnobData> { None }
    fn as_graph_data(&self) -> Option<&UIGraphData> { None }
    fn as_btn_data(&self) -> Option<&UIBtnData> { None }
    fn value_id(&self) -> usize;
//    fn size(&self) -> UIElementSize { UIElementSize::Normal };
}
