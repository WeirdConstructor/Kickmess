use crate::ui::constants::*;
use crate::ui::draw_cache::DrawCache;

#[derive(Debug, Clone, Copy)]
pub struct ActiveZone {
    pub id:      usize,
    pub subtype: usize,
    pub x:       f64,
    pub y:       f64,
    pub w:       f64,
    pub h:       f64,
}

pub enum ElementState {
    Active(f64),
    Hover(f64),
    Disabled(f64),
    TextEdit(String,u32),
}

pub enum Element {
    //   id,            label_idx
    Knob(usize,         usize),
    SmallKnob(usize,    usize),
    Toggle(usize,       usize),
}

pub enum Connector {
    Down,
    Right,
}

impl ActiveZone {
    pub fn from_rect(xo: f64, yo: f64, subtype: usize, r: (f64, f64, f64, f64)) -> Self {
        Self {
            id: 0,
            subtype,
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

pub struct Painter {
    pub zones: Vec<ActiveZone>,
    cache: DrawCache,
}

impl Painter {
    pub fn new() -> Self {
        Self {
            zones: vec![],
            cache: DrawCache::new(),
        }
    }

    pub fn start_redraw(&mut self)
    {
        self.zones.clear();
    }

    pub fn done_redraw(&mut self)
    {
    }

    fn add_active_zone(&mut self, id: usize, mut z: ActiveZone)
    {
        z.id = id;
        self.zones.push(z);
    }
}
