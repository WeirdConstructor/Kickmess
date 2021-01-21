// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

/* Parts of this file are translated from LMMS under GPLv2-or-later
 * into this project, which is GPLv3-or-later.
 *
 * DspEffectLibrary.h, kicker.cpp, KickerOsc.h
 *
 * Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
 * Copyright (c) 2014 grejppi <grejppi/at/gmail.com>
 * Copyright (c) 2020-2021 Weird Constructor <weirdconstructor/at/gmail.com>
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * General Public License for more details.
 *
 * You should have received a copy of the GNU General Public
 * License along with this program (see COPYING); if not, write to the
 * Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301 USA.
 *
 */

use crate::proc::*;
use crate::helpers::*;
use crate::env::*;
use crate::param_model::*;
use crate::filter::{MoogFilter, FilterInputParams};
use crate::oscillator::{UnisonBlep, FMOscillator, OscillatorInputParams};
use crate::log::Log;

use crate::MAX_BLOCKSIZE;
const PI2 : f64 = std::f64::consts::PI * 2.0;

struct F1Params<'a>(&'a ParamModel<'a>);
struct O1Params<'a, 'b>(&'a ParamModel<'a>, &'b f64);

impl<'a> FilterInputParams for F1Params<'a> {
    fn freq(&self)  -> f32 { self.0.f1_cutoff() }
    fn q(&self)     -> f32 { self.0.f1_res() }
    fn typ(&self)   -> f32 { self.0.f1_type() }
    fn drive(&self) -> f32 { self.0.f1_drive() }
}

impl<'a, 'b> OscillatorInputParams for O1Params<'a, 'b> {
    fn freq(&self)          -> f32 { *self.1 as f32 }
    fn waveform(&self)      -> f32 { self.0.o1_waveform() }
    fn pulse_width(&self)   -> f32 { self.0.o1_pw() }
    fn unison(&self)        -> f32 { self.0.o1_unison() }
    fn detune(&self)        -> f32 { self.0.o1_detune() }

    fn op1_ratio(&self)     -> f32 { self.0.o1fm_ratio() }
    fn op2_freq(&self)      -> f32 { self.0.o2fm_freq() }
    fn op1_self(&self)      -> f32 { self.0.o1fm_self() }
    fn op2_self(&self)      -> f32 { self.0.o2fm_self() }
    fn op1_op2(&self)       -> f32 { self.0.o1fm_o2_mod() }
    fn op2_op1(&self)       -> f32 { self.0.o2fm_o1_mod() }
}

pub struct OpKickmess {
    id:              usize,

    cur_f_start:     f64,
    cur_f_end:       f64,

    init_note_freq:  f64,
    note_freq:       f64,
    cur_phase:       f32,
    srate:           f32,

    rng:             RandGen,
    f_env:           REnv,
    release:         REnv,
    filter1:         MoogFilter,
    oscillator1:     UnisonBlep,
    fm_oscillator:   FMOscillator,
}

impl OpKickmess {
    fn next_sine_sample(&mut self, params: &ParamModel) -> f64 {
        let s =
            fast_sin(
                (self.cur_phase as f64
                 + (0.25 * (params.phase_offs() as f64)))
                * PI2);
        self.cur_phase +=
            (self.note_freq / (self.srate as f64)) as f32;
        s
    }
}

impl MonoProcessor for OpKickmess {
    fn init_params(ps: &mut ParamSet, public_ps: &mut ParamSet) {
        ParamModel::init_public_set(public_ps);
        ParamModel::init_private_set(ps);
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.srate = sr;
        self.release.set_sample_rate(sr);
        self.f_env.set_sample_rate(sr);
        self.filter1.set_sample_rate(sr);
        self.oscillator1.set_sample_rate(sr);
        self.fm_oscillator.set_sample_rate(sr);
    }

    fn process(&mut self, params: &SmoothParameters, proc_offs: usize, out: &mut [f32], log: &mut Log) {
        let block_params = ParamModel::new(params.get_frame(0));
        self.f_env.set_release(block_params.f_env_release());
        self.release.set_release(block_params.env_release());

        for (offs, os) in out.iter_mut().enumerate() {
            let params = ParamModel::new(params.get_frame(offs));
            let block_offs = offs + proc_offs;

            let mut kick_sample : f64 = 0.0;

            if let EnvPos::Running(pos, env_value) = self.f_env.next(block_offs) {
                if pos == 0 {
                    self.release.reset();
                    self.filter1.reset();
                    self.oscillator1.reset();
                    self.fm_oscillator.reset();

                    self.cur_phase = 0.0;

                    if params.freq_note_start() >= 0.5 {
                        self.cur_f_start = self.init_note_freq as f64;
                    } else {
                        self.cur_f_start = params.freq_start() as f64;
                    }

                    if params.freq_note_end() >= 0.5 {
                        self.cur_f_end = self.init_note_freq as f64;
                    } else {
                        self.cur_f_end = params.freq_end() as f64;
                    }

                    self.note_freq = self.cur_f_start as f64;
                }

                let amp_gain : f64 = 1.0 - env_value.powf(params.env_slope() as f64);

                let sine = self.next_sine_sample(&params);

                let noise =
                    (((self.rng.next_open01() * 2.0) - 1.0)
                     * amp_gain * amp_gain)
                    .max(-0.99).min(0.99);

                let mut s = lerp64(params.noise() as f64, sine, noise);

                kick_sample = s * amp_gain * params.gain() as f64;

                kick_sample +=
                    (params.o1_gain()
                     * amp_gain as f32
                     * self.oscillator1.next(&O1Params(&params, &self.note_freq))) as f64;

                kick_sample +=
                    (params.o2fm_gain()
                     * amp_gain as f32
                     * self.fm_oscillator.next(&O1Params(&params, &self.note_freq))) as f64;

//                if kick_sample.abs() > 0.99 {
//                    log.log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//                        use std::io::Write;
//                        write!(bw, "ks: {:9.6}", kick_sample).unwrap();
//                    });
//                }
//
                if params.f1_on() > 0.5 {
                    kick_sample =
                        self.filter1.next(kick_sample as f32, &F1Params(&params), log)
                        as f64;
                }

                if params.dist_on() > 0.5 {
                    let thres =
                        lerp(
                            env_value as f32,
                            params.dist_start(),
                            params.dist_end());

                    kick_sample =
                        f_distort(1.0, thres, kick_sample as f32) as f64;
                }

                kick_sample *= params.main_gain() as f64;

                let freq_change : f64 =
                    (self.cur_f_start - self.cur_f_end) as f64
                    * (1.0 - env_value.powf(params.freq_slope() as f64));

                self.note_freq = self.cur_f_end as f64 + freq_change;
            }

            let release_env_gain =
                match self.release.next(block_offs) {
                    EnvPos::Off => 1.0,
                    EnvPos::Running(_, value) => {
                        let gain : f64 = 1.0 - value.powf(0.5);
                        gain
                    },
                    EnvPos::End => {
                        self.f_env.reset();
                        self.release.reset();
                        0.0
                    }
                };

            *os += (kick_sample * release_env_gain) as f32;
        }
    }
}

impl MonoVoice for OpKickmess {
    fn new() -> Self {
        Self {
            id:              0,

            cur_f_start:     0.0,
            cur_f_end:       0.0,

            init_note_freq:  0.0,
            note_freq:       0.0,
            cur_phase:       0.0,
            srate:           0.0,

            rng:             RandGen::new(),
            f_env:           REnv::new(),
            release:         REnv::new(),
            filter1:         MoogFilter::new(),
            oscillator1:     UnisonBlep::new(10),
            fm_oscillator:   FMOscillator::new(),
        }
    }

    fn start_note(&mut self, id: usize, offs: usize, freq: f32, _vel: f32) {
        self.id             = id;
        self.init_note_freq = freq as f64;
        self.f_env.trigger(offs);

        // println!("{} freq: {:5.3}, offs: {}",
        //          self.id, self.init_note_freq, offs);
    }

    fn id(&self) -> usize { self.id }

    fn end_note(&mut self, offs: usize) {
        if self.f_env.active() {
            self.release.trigger(offs);
        }
    }

    fn is_playing(&self) -> bool {
        self.f_env.active()
        || self.release.active()
    }

    fn in_release(&self) -> bool {
        self.release.active()
    }
}

