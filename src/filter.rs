use crate::helpers::*;
use crate::log::Log;

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
    fn freq(&self)  -> f32;
    fn q(&self)     -> f32;
    fn typ(&self)   -> f32;
    fn drive(&self) -> f32;
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
        let q = 1.0 - params.q();
        self.low += self.f * self.band;
        let high = q * (input - self.band) - self.low;
        self.band += self.f * high;

        let ftype = params.typ();

        if      ftype < 0.25 { self.low }
        else if ftype < 0.50 { high }
        else if ftype < 0.75 { self.band }
        else                 { self.low + high } // notch
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoogFilter {
    srate:       f32,
    b0:          f32,
    b1:          f32,
    b2:          f32,
    b3:          f32,
    b4:          f32,
    cnt:         usize,
}

// From https://www.musicdsp.org/en/latest/Filters/25-moog-vcf-variation-1.html
// Author or source: CSound source code, Stilson/Smith CCRMA paper.,
//                   Paul Kellett version
// Type: 24db resonant lowpass
impl MoogFilter {
    pub fn new() -> Self {
        MoogFilter {
            srate:       44100.0, // is overwritten by set_sample_rate() anyways!
            b0:          0.0,
            b1:          0.0,
            b2:          0.0,
            b3:          0.0,
            b4:          0.0,
            cnt:         0,
        }
    }

    pub fn reset(&mut self) {
        self.b0 = 0.0;
        self.b1 = 0.0;
        self.b2 = 0.0;
        self.b3 = 0.0;
        self.b4 = 0.0;
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate;
    }

    pub fn next<P: FilterInputParams>(&mut self, mut input: f32, params: &P, log: &Log) -> f32 {
        self.cnt += 1;
        let freq = (params.freq()) / (self.srate * 0.5);

        let q = 1.0 - freq;
        let p = freq + 0.8 * freq * q;
        let f = p + p - 1.0;
        let q = params.q() * (1.0 + 0.5 * q * (1.0 - q + 5.6 * q * q));

        let mut input = quickTanh(params.drive() * input);

        input -= q * self.b4;    // feedback
        let t1 = self.b1; self.b1 = (input   + self.b0) * p - self.b1 * f;
        let t2 = self.b2; self.b2 = (self.b1 +      t1) * p - self.b2 * f;
        let t1 = self.b3; self.b3 = (self.b2 +      t2) * p - self.b3 * f;
                          self.b4 = (self.b3 +      t1) * p - self.b4 * f;

        // clipping
        self.b4 = self.b4 - self.b4 * self.b4 * self.b4 * 0.166667;
        // clamp to keep feedback run-aways in check!
//        self.b4 = f_distort(1.0, 1.0, self.b4); // .max(-1.0).min(1.0);
        self.b4 = self.b4.max(-1.5).min(1.5);
        self.b0 = input;

        if input.abs() > 0.99 {
            log.log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
                use std::io::Write;
                write!(bw, "fil: b0={:6.3} b3={:6.3} b4={:6.3} in={:6.3}", self.b0, self.b3, self.b4, input).unwrap();
            });
        }

        let ftype = params.typ();
        if      ftype < 0.33 { self.b4 }                   // lowpass
        else if ftype < 0.66 { input - self.b4 }           // highpass
        else                 { 3.0 * (self.b3 - self.b4) } // bandpass
    }
}

pub struct SvfSimperFilter {
    srate: f32,
    ic1eq: f32,
    ic2eq: f32
}

// Example taken from baseplug (Rust crate) example svf_simper.rs:
// implemented from https://cytomic.com/files/dsp/SvfLinearTrapOptimised2.pdf
// thanks, andy!
impl SvfSimperFilter {
    pub fn new() -> Self {
        SvfSimperFilter {
            srate: 44100.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate;
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    #[inline]
    pub fn next<P: FilterInputParams>(&mut self, mut input: f32, params: &P) -> f32 {
        let g = (std::f32::consts::PI * (params.freq() / self.srate)).tan();
        let k = 2f32 - (1.9f32 * params.q().min(1f32).max(0f32));

        let a1 = 1.0 / (1.0 + (g * (g + k)));
        let a2 = g * a1;
        let a3 = g * a2;

        let input = quickTanh(params.drive() * input);

        let v3 = input - self.ic2eq;
        let v1 = (a1 * self.ic1eq) + (a2 * v3);
        let v2 = self.ic2eq + (a2 * self.ic1eq) + (a3 * v3);

        self.ic1eq = (2.0 * v1) - self.ic1eq;
        self.ic2eq = (2.0 * v2) - self.ic2eq;

        v2
    }
}

// translated from Odin 2 Synthesizer Plugin
// Copyright (C) 2020 TheWaveWarden
// under GPLv3 or any later
pub struct DCBlockFilter {
    xm1:    f64,
    ym1:    f64,
    R:      f64,
}

impl DCBlockFilter {
    pub fn new() -> Self {
        Self {
            xm1: 0.0,
            ym1: 0.0,
            R:   0.995,
        }
    }

    pub fn reset(&mut self) {
        self.xm1 = 0.0;
        self.ym1 = 0.0;
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.R = 0.995;
        if srate > 90000.0 {
            self.R = 0.9965;
        } else if srate > 120000.0 {
            self.R = 0.997;
        }
    }

    pub fn next(&mut self, input: f32) -> f32 {
        let y = input as f64 - self.xm1 + self.R * self.ym1;
        self.xm1 = input as f64;
        self.ym1 = y;
        y as f32
    }
}
