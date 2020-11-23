/* Parts of this file are translated from LMMS under GPLv2-or-later
 * into this project, which is GPLv3-or-later.
 *
 * DspEffectLibrary.h, kicker.cpp, KickerOsc.h
 *
 * Copyright (c) 2006-2014 Tobias Doerffel <tobydox/at/users.sourceforge.net>
 * Copyright (c) 2014 grejppi <grejppi/at/gmail.com>
 * Copyright (c) 2020 Weird Constructor <weirdconstructor/at/gmail.com>
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

macro_rules! opKickmessParams {
    ($x: ident) => {
        // ($idx: literal, $id: ident, $type: ident, $init: expr, $param: expr,
        //  $min: expr, $max: expr, $default: expr, $name: literal)
        $x!{0,  freq_start,      f32, 0.0, Param::Freq1,      5.0,   3000.0, 150.0, "Start Freq."}
        $x!{1,  freq_end,        f32, 0.0, Param::Freq2,      5.0,   2000.0,  40.0, "End Freq."}
        $x!{2,  length,          f32, 0.0, Param::Decay1,     5.0,   5000.0, 440.0, "Length"}
        $x!{3,  dist_start,      f32, 0.0, Param::Dist1,      0.0,   100.0,    0.8, "Dist. Start"}
        $x!{4,  dist_end,        f32, 0.0, Param::Dist2,      0.0,   100.0,    0.8, "Dist. End"}
        $x!{5,  dist_gain,       f32, 0.0, Param::Gain1,      0.1,   5.0,      1.0, "Dist. Gain"}
        $x!{6,  env_slope,       f32, 0.0, Param::Env1,       0.01,  1.0,    0.163, "Env. slope"}
        $x!{7,  freq_slope,      f32, 0.0, Param::Release1,   0.001, 1.0,     0.06, "Freq. slope"}
        $x!{8,  noise,           f32, 0.0, Param::Noise1,     0.0,   1.0,      0.0, "Noise"}
        $x!{9,  freq_note_start, f32, 0.0, Param::S1,         0.0,   1.0,      1.0, "Start from note"}
        $x!{10, release,         f32, 0.0, Param::Release2,   1.0,1000.0,      5.0, "Env Release"}
    }
}

macro_rules! defineFields {
    ($idx: literal, $id: ident, $type: ident, $init: expr, $param: expr,
     $min: expr, $max: expr, $default: expr, $name: literal) => {
        $id : $type,
    }
}


macro_rules! defineIdxConstants {
    ($idx: literal, $id: ident, $type: ident, $init: expr, $param: expr,
     $min: expr, $max: expr, $default: expr, $name: literal) => {
        const $id : usize = $idx;
    }
}

opKickmessParams!{defineIdxConstants}

pub struct OpKickmess {
    freq_start:      f32,
    freq_end:        f32,
    dist_start:      f32,
    dist_end:        f32,
    dist_gain:       f32,
    env_slope:       f32,
    noise:           f32,
    freq_slope:      f32,
    freq_note_start: f32,

    rng:             RandGen,
    attack:          REnv,
    release:         REnv,
    srate:           f32,

    note_freq:       f64,
    cur_phase:       f32,
}

impl OpKickmess {
    pub fn new() -> Self {
        Self {
            freq_start:      0.0,
            freq_end:        0.0,
            dist_start:      0.0,
            dist_end:        0.0,
            dist_gain:       0.0,
            env_slope:       0.0,
            noise:           0.0,
            freq_slope:      0.0,
            freq_note_start: 0.0,

            rng:             RandGen::new(),
            attack:          REnv::new(),
            release:         REnv::new(),

            note_freq:       0.0,
            srate:           0.0,
            cur_phase:       0.0,
        }
    }
}

impl MonoProcessor for OpKickmess {
    fn init_params(ps: &mut ParamSet) {
        ps.add(ParamDefinition::from(Param::Freq1,      5.0,   3000.0, 150.0, "Start Freq."));
        ps.add(ParamDefinition::from(Param::Freq2,      5.0,   2000.0,  40.0, "End Freq."));
        ps.add(ParamDefinition::from(Param::Decay1,     5.0,   5000.0, 440.0, "Length"));
        ps.add(ParamDefinition::from(Param::Dist1,      0.0,   100.0,    0.8, "Dist. Start"));
        ps.add(ParamDefinition::from(Param::Dist2,      0.0,   100.0,    0.8, "Dist. End"));
        ps.add(ParamDefinition::from(Param::Gain1,      0.1,   5.0,      1.0, "Dist. Gain"));
        ps.add(ParamDefinition::from(Param::Env1,       0.01,  1.0,    0.163, "Env. slope"));
        ps.add(ParamDefinition::from(Param::Release1,   0.001, 1.0,     0.06, "Freq. slope"));
        ps.add(ParamDefinition::from(Param::Noise1,     0.0,   1.0,      0.0, "Noise"));
        ps.add(ParamDefinition::from(Param::S1,         0.0,   1.0,      1.0, "Start from note"));
        ps.add(ParamDefinition::from(Param::Release2,   1.0,1000.0,      5.0, "Env Release"));
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.srate = sr;
        self.release.set_sample_rate(sr);
        self.attack.set_sample_rate(sr);
    }

    fn read_params(&mut self, ps: &ParamSet, pp: &dyn ParamProvider) {
        self.freq_start       = ps.get( 0, pp);
        self.freq_end         = ps.get( 1, pp);
        self.attack.set_release(ps.get( 2, pp));
        self.dist_start       = ps.get( 3, pp);
        self.dist_end         = ps.get( 4, pp);
        self.dist_gain        = ps.get( 5, pp);
        self.env_slope        = ps.get( 6, pp);
        self.freq_slope       = ps.get( 7, pp);
        self.noise            = ps.get( 8, pp);
        self.freq_note_start  = ps.get( 9, pp);
        self.release.set_release(ps.get(10, pp));

        self.noise = self.noise * self.noise;
    }

    fn process(&mut self, l: &mut dyn Channel) {
        let has_dist_env = (self.dist_start - self.dist_end).abs() > 0.0001;

        l.process(&mut |_i: &[f32], o: &mut [f32]| {
            for (offs, os) in o.iter_mut().enumerate() {
                let mut kick_sample : f64 = 0.0;

                if let EnvPos::Release(pos, env_value) = self.attack.next(offs) {
                    if pos == 0 {
                        self.release.reset();
                        self.cur_phase = 0.0;
                    }

                    let gain : f64 = 1.0 - env_value.powf(self.env_slope as f64);

                    // const sample_t s =
                    //   ( Oscillator::sinSample( m_phase ) * ( 1 - m_noise ) )
                    //   + ( Oscillator::noiseSample( 0 ) * gain * gain * m_noise );
                    let mut s =
                        fast_sin(self.cur_phase as f64 * PI2)
                        * (1.0_f64 - self.noise as f64)
                        ; // TODO: + rng.noise...

                    s += self.rng.next_open01() * gain * gain * self.noise as f64;

                    kick_sample = s * gain;

                    // // update distortion envelope if necessary
                    // if( m_hasDistEnv && m_counter < m_length )
                    // {
                    // 	    float thres = linearInterpolate( m_distStart, m_distEnd, m_counter / m_length );
                    // 	    m_FX.leftFX().setThreshold( thres );
                    // 	    m_FX.rightFX().setThreshold( thres );
                    // }
                    // m_FX.nextSample( buf[frame][0], buf[frame][1] );

                    if has_dist_env {
                        let thres = p2range(env_value as f32, self.dist_start, self.dist_end);
                        kick_sample = f_distort(self.dist_gain, thres, kick_sample as f32) as f64;
                    }


                    // m_phase += m_freq / sampleRate;
                    self.cur_phase +=
                        (self.note_freq / (self.srate as f64)) as f32;

                    // const double change =
                    //  ( m_counter < m_length )
                    //      ? ( ( m_startFreq - m_endFreq )
                    //          * ( 1 - fastPow( m_counter / m_length, m_slope ) ) )
                    //      : 0;
                    let change : f64 =
                        if env_value <= 1.0 {
                            (self.freq_start - self.freq_end) as f64
                            * (1.0 - env_value.powf(self.freq_slope as f64))
                        } else {
                            0.0
                        };

                    // m_freq = m_endFreq + change;
                    self.note_freq = self.freq_end as f64 + change;
                }

                let release_env_gain =
                    match self.release.next(offs) {
                        EnvPos::Off => 1.0,
                        EnvPos::Release(_, value) => {
                            let gain : f64 = 1.0 - value.powf(0.5);
                            gain
                        },
                        EnvPos::End => {
                            self.attack.reset();
                            self.release.reset();
                            0.0
                        }
                    };

                *os += (kick_sample * release_env_gain) as f32;
            }
        });
    }
}

impl MonoVoice for OpKickmess {
    fn start_note(&mut self, offs: usize, freq: f32, _vel: f32) {
        self.attack.trigger(offs);

        if self.freq_note_start >= 0.5 {
            self.note_freq = freq as f64;
        } else {
            self.note_freq = self.freq_start as f64;
        }
    }

    fn end_note(&mut self, offs: usize) {
        if self.attack.active() {
            self.release.trigger(offs);
        }
    }

    fn is_playing(&self) -> bool {
        self.attack.active()
        || self.release.active()
    }

    fn in_release(&self) -> bool {
        self.release.active()
    }
}

