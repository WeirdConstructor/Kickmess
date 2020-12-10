use crate::ui::painting::*;
use crate::ui::constants::*;
use crate::ui::element::{UIElement, UIElementData};

pub struct DrawCache {
    surf:       Vec<Option<cairo::Surface>>,
    elements:   Vec<Box<dyn UIElement>>,
}

impl DrawCache {
    pub fn new() -> Self {
        Self {
            surf:       vec![],
            elements:   vec![],
        }
    }

    pub fn push_element(&mut self, el: Box<dyn UIElement>) {
        self.surf.push(None);
        self.elements.push(el);
    }

    pub fn size_of(&self, idx: usize) -> (f64, f64) {
        self.elements.get(idx).unwrap().size()
    }

    pub fn draw_data(&mut self, cr: &cairo::Context, x: f64, y: f64,
                     idx: usize, hover_style: bool,
                     data: &dyn UIElementData, value: f64, val_s: &str) {
        self.elements.get(idx)
            .unwrap()
            .draw_value(cr, x, y, hover_style, data, value, val_s);
    }

    pub fn define_active_zones(&self, x: f64, y: f64, elem_data: &dyn UIElementData,
                               idx: usize, f: &mut dyn FnMut(ActiveZone)) {

        self.elements.get(idx).unwrap().define_active_zones(x, y, elem_data, f);
    }

    pub fn draw_bg(&mut self, cr: &cairo::Context, x: f64, y: f64, idx: usize) {
        let element = self.elements.get(idx).unwrap();

        let (elem_w, elem_h) = element.size();

        if let None = self.surf[idx] {
            let surf = cr.get_target().create_similar_image(
                cairo::Format::ARgb32,
                elem_w as i32,
                elem_h as i32).expect("Createable new img surface");
            self.surf[idx] = Some(surf);
            let cr =
                cairo::Context::new(
                    self.surf[idx].as_mut().unwrap());
            element.draw_bg(&cr);
        }

        let surf = &self.surf[idx].as_ref().unwrap();

        cr.save();
        cr.set_source_surface(surf, x, y);
        cr.paint();
    }
}

