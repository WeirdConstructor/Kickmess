use crate::ui::constants::*;
use crate::ui::draw_cache::DrawCache;

pub const AZ_COARSE_DRAG : i8 = 0;
pub const AZ_FINE_DRAG   : i8 = 1;
pub const AZ_MOD_SELECT  : i8 = 2;
pub const AZ_TOGGLE      : i8 = 3;

#[derive(Debug, Clone, Copy)]
pub struct ActiveZone {
    pub id:      usize,
    pub subtype: usize,
    pub x:       f64,
    pub y:       f64,
    pub w:       f64,
    pub h:       f64,
}

#[derive(Debug, Clone, Copy)]
pub enum HLStyle {
    None,
    Hover(i8),
    ModTarget,
    HoverModTarget,
}

impl ActiveZone {
    pub fn from_rect(xo: f64, yo: f64, subtype: i8, r: (f64, f64, f64, f64)) -> Self {
        Self {
            id: 0,
            subtype: subtype as usize,
            x: r.0 + xo,
            y: r.1 + yo,
            w: r.2,
            h: r.3,
        }
    }

    pub fn is_inside(&self, x: f64, y: f64) -> bool {
           x >= self.x && x <= (self.x + self.w)
        && y >= self.y && y <= (self.y + self.h)
    }
}

pub trait Painter {
    fn path_fill(&self, color: (f64, f64, f64), segments: &[(f64, f64)]);
    fn path_stroke(&self, width: f64, color: (f64, f64, f64), segments: &[(f64, f64)]);
    fn rect_fill(&self, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64);
    fn rect_stroke(&self, width: f64, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64);
    fn label(&self, size: f64, align: i8, color: (f64, f64, f64), x: f64, y: f64, w: f64, h: f64, text: &str);
}
