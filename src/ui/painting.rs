#[derive(Debug, Clone, Copy)]
pub struct ActiveZone {
    pub id:  usize,
    pub x:   f64,
    pub y:   f64,
    pub w:   f64,
    pub h:   f64,
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
    fn from_rect(xo: f64, yo: f64, r: (f64, f64, f64, f64)) -> Self {
        Self {
            id: 0,
            x: r.0 + xo,
            y: r.1 + yo,
            w: r.2,
            h: r.3,
        }
    }

    fn is_inside(&self, x: f64, y: f64) -> bool {
           x >= self.x && x <= (self.x + self.w)
        && y >= self.y && y <= (self.y + self.h)
    }
}

pub struct Painter {
    pub zones: Vec<ActiveZone>,
}

impl Painter {
    pub fn new() -> Self {
        Self {
            zones: vec![],
        }
    }

    pub fn paint_element_hbox(&mut self, name: &str, x: usize, y: usize, elements: &[Element], states: &[ElementState])
    {
        let mut w =
            elements.iter().fold(0.0, |w, e| {
                w + 2.0 * UI_PADDING + match e {
                    Element::Knob(_, _)      => UI_ELEM_N_W,
                    Element::SmallKnob(_, _) => UI_ELEM_N_W * 0.8,
                    Element::Toggle(_, _)    => UI_ELEM_N_W,
                }
            });


        let mut h = UI_BOX_H + 2.0 * UI_PADDING;

        let x = x as f64 * (UI_ELEM_N_W + UI_MARGIN);
        let y = y as f64 * (UI_ELEM_N_H + UI_MARGIN);

        self.cr.save();
        self.cr.set_line_width(1.0);

        self.cr.set_source_rgb(0.29, 0.29, 0.29);
        self.cr.rectangle(x, y, w, h);
        self.cr.fill();

        self.cr.set_source_rgb(0.54, 0.54, 0.54);
        self.cr.rectangle(
            x + UI_PADDING,
            y + UI_PADDING,
            w - 2.0 * UI_PADDING,
            h - 2.0 * UI_PADDING);
        self.cr.fill();
        self.cr.restore();

        let mut x = x;
        let mut y = y + UI_ELEM_TXT_H;
        for e in elements.iter() {
            x += UI_PADDING;
            match e {
                Element::Knob(id, _) => {
                    let az = self.cache.draw_knob_bg(self.cr, x, y, "SFreq");
                    self.add_active_zone(*id, az);
                    self.cache.draw_knob_data(self.cr, x, y, 0.75, "0.75");
                    x += UI_ELEM_N_W;
                },
                _ => {}
            }
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
