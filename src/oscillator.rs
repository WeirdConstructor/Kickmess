use crate::helpers;

pub trait OscillatorInputParams{
    fn freq(&self)          -> f32;
    fn waveform(&self)      -> f32;
    fn pulse_width(&self)   -> f32;
}

pub struct BlitOsc {
    srate:      f64,
    phase:      f64,
    integral:   f64,
}

impl BlitOsc {
    pub fn new() -> Self {
        Self {
            srate:      0.0,
            phase:      0.0,
            integral:   0.0,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate as f64;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
        self.integral = 0.0;
    }

    pub fn next<P: OscillatorInputParams>(&mut self, params: &P) -> f32 {
        let phase_max : f64 =
            (self.srate * 0.5) / (params.freq() as f64); // helpers::note_to_freq(note);
        let dc_offs : f64 = -0.498 / phase_max;

        let pulse_width = params.pulse_width() as f64;
        //d// println!("FO {} {} {}", self.phase, phase_max, pulse_width);
        let mut phase2 : f64 =
            ((self.phase + 2.0 * phase_max * pulse_width)
             % (phase_max * 2.0)) - phase_max;
        self.phase = (self.phase + 1.0) % (phase_max * 2.0);
        let mut tmp_phase : f64 = self.phase - phase_max;

        let epsilon : f64 = 0.0000001;
        let blit1 : f64 =
            if tmp_phase > epsilon || tmp_phase < -epsilon {
                tmp_phase *= 3.141592;
                //d// println!("IN: {} => {}", tmp_phase,  helpers::fast_sin(tmp_phase));
                helpers::fast_sin(tmp_phase) / tmp_phase
            } else {
                1.0
            };
        let blit2 : f64 =
            if phase2 > epsilon || phase2 < -epsilon {
                phase2 *= 3.141592;
                helpers::fast_sin(phase2) / phase2
            } else {
                1.0
            };

        //d// println!("B1={} B2={}", blit1, blit2);

        let waveform = params.waveform() as f64;

        self.integral =
            0.998 * self.integral
            + dc_offs * (1.0 - waveform)
            + blit1
            - blit2 * waveform;

//        println!("NEX: {}", self.integral);
        self.integral as f32
    }
}


pub struct PolyBlep {
    srate:      f64,
    phase:      f64,
    t:          f64,
}

fn sqr(x: f64) -> f64 { x * x }

fn blep(t: f64, dt: f64) -> f64 {
    if t < dt {
        let t = t / dt;
        2. * t - sqr(t) - 1.

    } else if t > (1. - dt) {
        let t = (t - 1.) / dt;
        sqr(t) + 2. * t + 1.

    } else {
        0.
    }
}

// https://dsp.stackexchange.com/questions/54790/polyblamp-anti-aliasing-in-c
// ?
// Derived from blep().
fn blamp(mut t: f64, dt: f64) -> f64 {
    if t < dt {
        t = t / dt - 1.0;
        -1.0 / 3.0 * sqr(t) * t

    } else if t > 1.0 - dt {
        t = (t - 1.0) / dt + 1.0;
        1.0 / 3.0 * sqr(t) * t

    } else {
        0.0
    }
}

impl PolyBlep {
    pub fn new() -> Self {
        Self {
            srate:      0.0,
            phase:      0.0,
            t:          0.0,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate as f64;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next_tri(&mut self, freq: f64) -> f64 {
        let t1 = (self.t + 0.25).fract();
        let t2 = (self.t + 0.75).fract();

        let mut y = self.t * 4.0;

        if y >= 3.0 {
            y -= 4.0;
        } else if y > 1.0 {
            y = 2.0 - y;
        }

        y += 4.0 * self.t * (blamp(t1, freq) - blamp(t2, freq));

        y
    }

    pub fn next<P: OscillatorInputParams>(&mut self, params: &P) -> f32 {
        let freq = (params.freq() / self.srate as f32) as f64;

        let wave = params.waveform();

        let s =
            if wave < 0.5 {
                // tri => square
                self.next_tri(freq)
            } else {
                // square => saw
                self.next_tri(freq)
            };

        self.t += freq;
        self.t = self.t.fract();

        s as f32
    }
}
