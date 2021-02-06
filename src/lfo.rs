use crate::helpers::*;

pub trait LFOInputParams {
    fn freq(&self)          -> f32;
    fn waveform(&self)      -> f32;
    fn pulse_width(&self)   -> f32;
    fn phase_offs(&self)    -> f32;
}

impl LFOInputParams for (f32, f32, f32, f32) {
    fn freq(&self)          -> f32 { self.0 }
    fn waveform(&self)      -> f32 { self.1 }
    fn pulse_width(&self)   -> f32 { self.2 }
    fn phase_offs(&self)    -> f32 { self.3 }
}

pub struct LFO {
    srate:       f64,
    phase:       f64,
    phase_offs:  f64,
}

impl LFO {
    pub fn new() -> Self {
        Self {
            srate:       0.0,
            phase:       0.0,
            phase_offs:  0.0,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate as f64;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next_sin(&mut self) -> f64 {
        fast_sin((self.phase + self.phase_offs).fract() * 2.0 * std::f64::consts::PI)
    }

    pub fn next_tri(&mut self) -> f64 {
        let value = -1.0 + (2.0 * (self.phase + self.phase_offs).fract());
        2.0 * (value.abs() - 0.5)
    }

    pub fn next_saw(&mut self) -> f64 {
        (2.0 * (self.phase + self.phase_offs).fract()) - 1.0
    }

    pub fn next_sqr(&mut self, pw: f64) -> f64 {
        if (self.phase + self.phase_offs).fract() < pw { 1.0 }
        else { -1.0 }
    }

    pub fn next<P: LFOInputParams>(&mut self, params: &P) -> f32 {
        let freq = params.freq() as f64;
        let phase_inc = freq / self.srate;
        self.phase_offs = params.phase_offs() as f64;

        let wave = params.waveform();

        let sample =
                 if wave < 0.25 { self.next_sin() }
            else if wave < 0.5  { self.next_tri() }
            else if wave < 0.75 { self.next_saw() }
            else                { self.next_sqr(params.pulse_width() as f64) };

        self.phase += phase_inc;
        self.phase = self.phase.fract();

        ((sample + 1.0) * 0.5) as f32
    }
}
