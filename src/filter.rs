use crate::helpers::*;

// Digital approx. of Chamberlin two-pole low pass.
// From: Author or source: Effect Deisgn Part 1, Jon Dattorro, J. Audio Eng. Soc.,
//                         Vol 45, No. 9, 1997 September
// Type: 12db resonant low/high/bandpass
// Seen on: https://www.musicdsp.org/en/latest/Filters/23-state-variable.html
// Originally translated from the C++ synthesizer "WaveSabre" by Logicoma
#[derive(Debug, Clone, Copy)]
pub struct SvfFilterOversampled {
    srate:       f32,
    last_input:  f32,
    low:         f32,
    band:        f32,
    f:           f32,
}

pub trait FilterInputParams {
    fn freq(&self) -> f32;
    fn q(&self)    -> f32;
    fn typ(&self) -> f32;
}

impl SvfFilterOversampled {
    pub fn new() -> Self {
        SvfFilterOversampled {
            srate:       44100.0, // is overwritten by set_sample_rate() anyways!
            last_input:  0.0,
            low:         0.0,
            band:        0.0,
            f:           0.0,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate;
    }

    pub fn next<P: FilterInputParams>(&mut self, input: f32, params: &P) -> f32 {
        let f =
            2.0 // 2.0 is what musicdsp suggests? but it does not suggest 0.5 on
                // the immediate output sample either.
            * fast_sin(
                (std::f64::consts::PI * params.freq() as f64)
                / (self.srate as f64 * 2.0)); // 2.0 because of double sampling?!
        self.f = f as f32;

        // We oversample here. But instead of throwing away the first
        // output sample, we kind of calculate that into the end output.
        // I don't really know what this is about, it's taken from WaveSabre,
        // but musicdsp and especially
        // http://www.earlevel.com/main/2003/03/02/the-digital-state-variable-filter/
        // say, that the output sample should just be thrown away.
        let ret =
            (  self.run(self.last_input + input, params) * 0.5
             + self.run(input, params))
            * 0.5;
        self.last_input = input;
        ret
    }

    fn run<P: FilterInputParams>(&mut self, input: f32, params: &P) -> f32 {
        self.low += self.f * self.band;
        let high = params.q() * (input - self.band) - self.low;
        self.band += self.f * high;

        let ftype = params.typ();

        if      ftype < 0.25 { self.low }
        else if ftype < 0.50 { high }
        else if ftype < 0.75 { self.band }
        else                 { self.low + high } // notch
    }
}
