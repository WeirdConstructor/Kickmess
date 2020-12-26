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

use crate::MAX_BLOCKSIZE;
const PI2 : f64 = std::f64::consts::PI * 2.0;

macro_rules! param_model {
    ($x: ident) => {
        $x!{public freq_start      exp no_smooth 0,   5.0,   3000.0, 150.0, "Start Freq."}
        $x!{public freq_end        exp no_smooth 1,   5.0,   2000.0,  40.0, "End Freq."}
        $x!{public f_env_release   exp no_smooth 2,   5.0,   5000.0, 440.0, "Length"}
        $x!{public dist_start      lin smooth    3,   0.0,   100.0,    0.8, "Dist. Start"}
        $x!{public dist_end        lin smooth    4,   0.0,   100.0,    0.8, "Dist. End"}
        $x!{public dist_gain       lin smooth    5,   0.1,   5.0,      1.0, "Dist. Gain"}
        $x!{public env_slope       lin smooth    6,   0.01,  1.0,    0.163, "Env. slope"}
        $x!{public freq_slope      lin smooth    7,   0.001, 1.0,     0.06, "Freq. slope"}
        $x!{public noise           exp smooth    8,   0.0,   1.0,      0.0, "Noise"}
        $x!{public freq_note_start lin no_smooth 9,   0.0,   1.0,      1.0, "Start from note"}
        $x!{public freq_note_end   lin no_smooth 10,  0.0,   1.0,      1.0, "End from note"}
        $x!{public env_release     lin no_smooth 11,  1.0,1000.0,      5.0, "Env Release"}
        $x!{public phase_offs      lin smooth    12,  0.0,   1.0,      0.0, "Click"}
        $x!{private phase_test     lin smooth    13,  0.0,   1.0,      0.0, "Click2"}
    }
}

struct ParamModel<'a> {
    v: &'a [f32],
}

macro_rules! param_impl_accessors {
    ($_:ident $name:ident $e:ident $s:ident $idx:expr, $($tt:tt)*) => {
        impl ParamModel<'_> {
            fn $name(&self) -> f32 { self.v[$idx] }
        }
    }
}

impl<'a> ParamModel<'a> {
    fn new(v: &'a [f32]) -> Self {
        Self { v }
    }

    fn init_public_set(ps: &mut ParamSet) {
        macro_rules! param_add_ps {
            (public $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $lbl:expr) => {
                ps.add(ParamDefinition::from($idx, $min, $max, $def, $lbl).$e().$s());
            };
            (private $($tt:tt)*) => {
            }
        }

        param_model!{param_add_ps}
    }

    fn init_private_set(ps: &mut ParamSet) {
        macro_rules! param_add_ps_priv {
            ($_:ident $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $lbl:expr) => {
                ps.add(ParamDefinition::from($idx, $min, $max, $def, $lbl).$e().$s());
            }
        }

        param_model!{param_add_ps_priv}
    }
}

param_model!{param_impl_accessors}


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
}

impl MonoProcessor for OpKickmess {
    fn init_params(ps: &mut ParamSet, public_ps: &mut ParamSet) {
//        public_ps.add2(ps, ParamDefinition::from(Param::Freq1,      5.0,   3000.0, 150.0, "Start Freq.").exp().no_smooth());
//        public_ps.add2(ps, ParamDefinition::from(Param::Freq2,      5.0,   2000.0,  40.0, "End Freq.").exp().no_smooth());
//        public_ps.add2(ps, ParamDefinition::from(Param::Decay1,     5.0,   5000.0, 440.0, "Length").exp().no_smooth());
//        public_ps.add2(ps, ParamDefinition::from(Param::Dist1,      0.0,   100.0,    0.8, "Dist. Start"));
//        public_ps.add2(ps, ParamDefinition::from(Param::Dist2,      0.0,   100.0,    0.8, "Dist. End"));
//        public_ps.add2(ps, ParamDefinition::from(Param::Gain1,      0.1,   5.0,      1.0, "Dist. Gain"));
//        public_ps.add2(ps, ParamDefinition::from(Param::Env1,       0.01,  1.0,    0.163, "Env. slope"));
//        public_ps.add2(ps, ParamDefinition::from(Param::Release1,   0.001, 1.0,     0.06, "Freq. slope"));
//        public_ps.add2(ps, ParamDefinition::from(Param::Noise1,     0.0,   1.0,      0.0, "Noise"));
//        public_ps.add2(ps, ParamDefinition::from(Param::S1,         0.0,   1.0,      1.0, "Start from note").no_smooth());
//        public_ps.add2(ps, ParamDefinition::from(Param::S2,         0.0,   1.0,      1.0, "End from note").no_smooth());
//        public_ps.add2(ps, ParamDefinition::from(Param::Release2,   1.0,1000.0,      5.0, "Env Release").no_smooth());
//        public_ps.add2(ps, ParamDefinition::from(Param::Phase1,     0.0,   1.0,      0.0, "Click"));
        ParamModel::init_public_set(public_ps);
        ParamModel::init_private_set(ps);
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.srate = sr;
        self.release.set_sample_rate(sr);
        self.f_env.set_sample_rate(sr);
    }

    fn process(&mut self, params: &SmoothParameters, proc_offs: usize, out: &mut [f32]) {
        let block_params = ParamModel::new(params.get_frame(0));
        self.f_env.set_release(block_params.f_env_release());
        self.release.set_release(block_params.env_release());

        for (offs, os) in out.iter_mut().enumerate() {
            let params = ParamModel::new(params.get_frame(offs));
            let block_offs = offs + proc_offs;

            let mut kick_sample : f64 = 0.0;

            if let EnvPos::Release(pos, env_value) = self.f_env.next(block_offs) {
                if pos == 0 {
                    self.release.reset();
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

                let gain : f64 = 1.0 - env_value.powf(params.env_slope() as f64);

                let mut s =
                    fast_sin((self.cur_phase as f64 + (params.phase_offs() as f64))
                             * PI2)
                    * (1.0_f64 - params.noise() as f64);

                s += self.rng.next_open01() * gain * gain * params.noise() as f64;

                kick_sample = s * gain;

                if (params.dist_start() - params.dist_end()).abs() > 0.0001 {
                    let thres = p2range(env_value as f32, params.dist_start(), params.dist_end());
                    kick_sample = f_distort(params.dist_gain(), thres, kick_sample as f32) as f64;
                }

                self.cur_phase +=
                    (self.note_freq / (self.srate as f64)) as f32;
//                    println!("nf: {:5.3}", self.note_freq);

                let change : f64 =
                    if env_value <= 1.0 {
                        (self.cur_f_start - self.cur_f_end) as f64
                        * (1.0 - env_value.powf(params.freq_slope() as f64))
                    } else {
                        0.0
                    };

                self.note_freq = self.cur_f_end as f64 + change;
            }

            let release_env_gain =
                match self.release.next(block_offs) {
                    EnvPos::Off => 1.0,
                    EnvPos::Release(_, value) => {
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

        }
    }

    fn start_note(&mut self, id: usize, offs: usize, freq: f32, _vel: f32) {
        self.id = id;
        self.init_note_freq = freq as f64;
        self.f_env.trigger(offs);

        println!("{} freq: {:5.3}, offs: {}",
                 self.id, self.init_note_freq, offs);
//                 self.cur_f_start,
//                 self.cur_f_end);
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

