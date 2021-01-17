use crate::helpers;

pub trait OscillatorInputParams{
    fn freq(&self)          -> f32;
    fn waveform(&self)      -> f32;
    fn pulse_width(&self)   -> f32;
}

pub struct PolyBlep {
    srate:       f64,
    phase:       f64,
    last_output: f64,
}

enum Waveform {
    Sin,
    Tri,
    Saw,
    Sqr,
}

fn sqr(x: f64) -> f64 { x * x }

// PolyBLEP by Tale
// (slightly modified)
// http://www.kvraudio.com/forum/viewtopic.php?t=375517
// from http://www.martin-finke.de/blog/articles/audio-plugins-018-polyblep-oscillator/
fn poly_blep(t: f64, dt: f64) -> f64 {
    if t < dt {
        let t = t / dt;
        2. * t - sqr(t) - 1.

    } else if t > (1.0 - dt) {
        let t = (t - 1.0) / dt;
        sqr(t) + 2. * t + 1.

    } else {
        0.
    }
}

impl PolyBlep {
    pub fn new() -> Self {
        Self {
            srate:       0.0,
            phase:       0.0,
            last_output: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate as f64;
    }

    pub fn reset(&mut self) {
        self.phase       = 0.0;
        self.last_output = 0.0;
    }

    pub fn next_sin(&mut self) -> f64 {
        crate::helpers::fast_sin(self.phase * 2.0 * std::f64::consts::PI)
    }

    pub fn next_tri(&mut self) -> f64 {
        let value = -1.0 + (2.0 * self.phase);
        2.0 * (value.abs() - 0.5)
    }

    pub fn next_saw(&mut self) -> f64 {
        (2.0 * self.phase) - 1.0
    }

    pub fn next_sqr(&mut self) -> f64 {
        if self.phase < 0.5 { 1.0 }
        else { -1.0 }
    }

    pub fn next<P: OscillatorInputParams>(&mut self, params: &P) -> f32 {
        let phase_inc = params.freq() as f64 / self.srate;

        let wave = params.waveform();

        let waveform =
            if wave < 0.25      { Waveform::Sin }
            else if wave < 0.5  { Waveform::Tri }
            else if wave < 0.75 { Waveform::Saw }
            else                { Waveform::Sqr };

        let sample =
            match waveform {
                Waveform::Sin => self.next_sin(),
                Waveform::Tri => {
                    let mut sample = self.next_sqr();
                    sample += poly_blep(self.phase, phase_inc);
                    sample -= poly_blep((self.phase + 0.5).fract(), phase_inc);

                    // leaky integrator: y[n] = A * x[n] + (1 - A) * y[n-1]
                    sample =
                        phase_inc * sample
                        + (1.0 - phase_inc) * self.last_output;
                    self.last_output = sample;
                    sample
                },
                Waveform::Saw => {
                    let mut sample = self.next_saw();
                    sample -= poly_blep(self.phase, phase_inc);
                    sample
                },
                Waveform::Sqr => {
                    let mut sample = self.next_sqr();
                    sample += poly_blep(self.phase, phase_inc);
                    sample -= poly_blep((self.phase + 0.5).fract(), phase_inc);
                    sample
                },
            };

        self.phase += phase_inc;
        self.phase = self.phase.fract();

        sample as f32
    }
}
