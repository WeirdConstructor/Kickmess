static FAST_COS_TAB_LOG2_SIZE : usize = 9;
static FAST_COS_TAB_SIZE : usize      = 1 << FAST_COS_TAB_LOG2_SIZE; // =512
static mut FAST_COS_TAB : [f64; 513] = [0.0; 513];

pub fn init_cos_tab() {
    for i in 0..(FAST_COS_TAB_SIZE+1) {
        let phase : f64 =
            (i as f64)
            * ((std::f64::consts::PI * 2.0)
               / (FAST_COS_TAB_SIZE as f64));
        unsafe {
            // XXX: note: mutable statics can be mutated by multiple
            //      threads: aliasing violations or data races
            //      will cause undefined behavior
            FAST_COS_TAB[i] = phase.cos();
        }
    }
}

pub fn fast_cos(mut x: f64) -> f64 {
    x = x.abs(); // cosine is symmetrical around 0, let's get rid of negative values

    // normalize range from 0..2PI to 1..2
    let phase_scale  = 1.0_f64 / (std::f64::consts::PI * 2.0_f64);
    let phase        = 1.0_f64 + x * phase_scale;

    let phase_as_u64 : u64 = unsafe { std::mem::transmute::<f64, u64>(phase) };//  phase.to_bits();
    let exponent     = (phase_as_u64 >> 52) - 1023;

    let fract_bits : u32  = 32 - FAST_COS_TAB_LOG2_SIZE as u32;
    let fract_scale  = 1 << fract_bits;
    let fract_mask   = fract_scale - 1;


    let significand  = ((phase_as_u64 << exponent) >> (52 - 32)) as u32;
    let index        = significand >> fract_bits;
    let fract : i32  = (significand as i32) & fract_mask;

    unsafe {
        // XXX: note: mutable statics can be mutated by multiple
        //      threads: aliasing violations or data races
        //      will cause undefined behavior
        let left         = FAST_COS_TAB[index as usize];
        let right        = FAST_COS_TAB[index as usize + 1];
        let fract_mix    = (fract as f64) * (1.0 / (fract_scale as f64));

        return left + (right - left) * fract_mix;
    }
}

pub fn fast_sin(x: f64) -> f64 {
    fast_cos(x - (std::f64::consts::PI / 2.0))
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RandGen {
    r: [u64; 2],
}

// Taken from xoroshiro128 crate under MIT License
// Implemented by Matthew Scharley (Copyright 2016)
// https://github.com/mscharley/rust-xoroshiro128
pub fn next_xoroshiro128(state: &mut [u64; 2]) -> u64 {
    let s0: u64     = state[0];
    let mut s1: u64 = state[1];
    let result: u64 = s0.wrapping_add(s1);

    s1 ^= s0;
    state[0] = s0.rotate_left(55) ^ s1 ^ (s1 << 14); // a, b
    state[1] = s1.rotate_left(36); // c

    result
}

// Taken from rand::distributions
// Licensed under the Apache License, Version 2.0
// Copyright 2018 Developers of the Rand project.
pub fn u64_to_open01(u: u64) -> f64 {
    use core::f64::EPSILON;
    let float_size         = std::mem::size_of::<f64>() as u32 * 8;
    let fraction           = u >> (float_size - 52);
    let exponent_bits: u64 = (1023 as u64) << 52;
    f64::from_bits(fraction | exponent_bits) - (1.0 - EPSILON / 2.0)
}

impl RandGen {
    pub fn new() -> Self {
        RandGen {
            r: [0x193a6754a8a7d469, 0x97830e05113ba7bb],
        }
    }

    pub fn next(&mut self) -> u64 {
        next_xoroshiro128(&mut self.r)
    }

    pub fn next_open01(&mut self) -> f64 {
        u64_to_open01(self.next())
    }
}

pub fn mix(v1: f32, v2: f32, mix: f32) -> f32 {
    v1 * (1.0 - mix) + v2 * mix
}

pub fn clamp(f: f32, min: f32, max: f32) -> f32 {
         if f < min { min }
    else if f > max { max }
    else            {   f }
}

pub fn square_135(phase: f64) -> f64 {
      fast_sin(phase)
    + fast_sin(phase * 3.0) / 3.0
    + fast_sin(phase * 5.0) / 5.0
}

pub fn square_35(phase: f64) -> f64 {
      fast_sin(phase * 3.0) / 3.0
    + fast_sin(phase * 5.0) / 5.0
}

pub fn note_to_freq(note: f64) -> f64 {
    440.0 * (2.0_f64).powf((note - 69.0) / 12.0)
}

// Signal distortion
// gain:        0.1 - 5.0       default = 1.0
// threshold:   0.0 - 100.0     default = 0.8
// i:           signal
fn f_distort(gain: f32, threshold: f32, i: f32) -> f32 {
    gain * (
        i * ( i.abs() + threshold )
        / ( i * i + (threshold - 1.0) * i.abs() + 1.0 ))
}

pub fn p2range(x: f32, a: f32, b: f32) -> f32 {
    (a * (1.0 - x)) + (b * x)
}

pub fn range2p(v: f32, a: f32, b: f32) -> f32 {
    (v - b) / (a - b)
}

