use crate::proc::*;
use crate::helpers::*;

pub struct Op_Kickmess {
    freq_start:      f32,
    freq_end:        f32,
    length_ms:       f32,
    dist_start:      f32,
    dist_end:        f32,
    dist_gain:       f32,
    env_slope:       f32,
    noise:           f32,
    phase:           f32,
    freq_slope:      f32,
    freq_note_start: bool,

    note_offs:       Option<usize>,
    note_freq:       f64,
    srate:           f32,
    counter:         usize,
    note_samples:    usize,
    cur_phase:       f32,
}

impl Op_Kickmess {
    pub fn new() -> Self {
        Self {
            freq_start:      0.0,
            freq_end:        0.0,
            length_ms:       0.0,
            dist_start:      0.0,
            dist_end:        0.0,
            dist_gain:       0.0,
            env_slope:       0.0,
            noise:           0.0,
            phase:           0.0,
            freq_slope:      0.0,
            freq_note_start: true,

            note_offs:       None,
            note_freq:       0.0,
            counter:         0,
            srate:           0.0,
            note_samples:    0,
            cur_phase:       0.0,
        }
    }
}

impl MonoProcessor for Op_Kickmess {
    fn init_params(ps: &mut ParamSet) {
        ps.add(ParamDefinition::from(Param::Freq1,      5.0,   1000.0, 150.0, "Start Freq."));
        ps.add(ParamDefinition::from(Param::Freq2,      5.0,   1000.0,  40.0, "End Freq."));
        ps.add(ParamDefinition::from(Param::Decay1,     5.0,   5000.0, 440.0, "Length"));
        ps.add(ParamDefinition::from(Param::Dist1,      0.0,   100.0,    0.8, "Dist. Start"));
        ps.add(ParamDefinition::from(Param::Dist2,      0.0,   100.0,    0.8, "Dist. End"));
        ps.add(ParamDefinition::from(Param::Gain1,      0.1,   5.0,      1.0, "Dist. Gain"));
        ps.add(ParamDefinition::from(Param::Env1,       0.01,  1.0,    0.163, "Env. slope"));
        ps.add(ParamDefinition::from(Param::Release1,   0.001, 1.0,     0.06, "Freq. slope"));
        ps.add(ParamDefinition::from(Param::Noise1,     0.0,   1.0,      0.0, "Noise"));
        ps.add(ParamDefinition::from(Param::Phase1,     0.0,   1.0,      0.4, "Click/Phase start"));
        ps.add(ParamDefinition::from(Param::S1,         0.0,   1.0,      1.0, "Start from note"));
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.srate = sr;
    }

    fn read_params(&mut self, ps: &ParamSet, pp: &dyn ParamProvider) {
        self.freq_start      = ps.get( 0, pp);
        self.freq_end        = ps.get( 1, pp);
        self.length_ms       = ps.get( 2, pp);
        self.dist_start      = ps.get( 3, pp);
        self.dist_end        = ps.get( 4, pp);
        self.dist_gain       = ps.get( 5, pp);
        self.env_slope       = ps.get( 6, pp);
        self.freq_slope      = ps.get( 7, pp);
        self.noise           = ps.get( 8, pp);
        self.phase           = ps.get( 9, pp);
        self.freq_note_start = ps.get(10, pp) >= 0.5;

        self.noise = self.noise * self.noise;
        self.phase = self.phase * 0.25;
    }

    fn process(&mut self, l: &mut dyn Channel) {

        l.process(&mut |i: &[f32], o: &mut [f32]| {
            let mut offs = 0;
            for (is, os) in i.iter().zip(o) {
                if let Some(offs) = self.note_offs {
                    println!("STARTED VOICE {} Hz AT {:?}", self.note_freq, self.note_offs);
                    self.note_offs = None;
                    self.counter += 1;
                } else if self.counter > 0 {
                    self.counter += 1;
                }

                if self.counter > 0 {
                    //	const double gain =
                    //      ( 1 - fastPow(( m_counter < m_length )
                    //                    ? m_counter / m_length
                    //                    : 1,
                    //                    m_env ) );
                    let len : f64 =
                        if (self.counter - 1) < self.note_samples {
                            ((self.counter - 1) as f64)
                            / (self.note_samples as f64)
                        } else {
                            1.0
                        };

                    let gain : f64 = 1.0 - len.powf(self.env_slope as f64);

                    // const sample_t s =
                    //   ( Oscillator::sinSample( m_phase ) * ( 1 - m_noise ) )
                    //   + ( Oscillator::noiseSample( 0 ) * gain * gain * m_noise );
                    let s =
                        fast_sin(self.cur_phase as f64) * (1.0_f64 - self.noise as f64)
                        ; // TODO: + rng.noise...

                    if offs == 0 {
                        println!("PHAS {} | {}", self.cur_phase, gain);
                    }

                    // buf[frame][0] = s * gain;
                    // buf[frame][1] = s * gain;
                    *os = (s * gain) as f32;

                    // // update distortion envelope if necessary
                    // if( m_hasDistEnv && m_counter < m_length )
                    // {
                    // 	    float thres = linearInterpolate( m_distStart, m_distEnd, m_counter / m_length );
                    // 	    m_FX.leftFX().setThreshold( thres );
                    // 	    m_FX.rightFX().setThreshold( thres );
                    // }
                    // m_FX.nextSample( buf[frame][0], buf[frame][1] );

                    // m_phase += m_freq / sampleRate;
                    self.cur_phase +=
                        (self.note_freq / (self.srate as f64)) as f32;

                    // const double change =
                    //  ( m_counter < m_length )
                    //      ? ( ( m_startFreq - m_endFreq )
                    //          * ( 1 - fastPow( m_counter / m_length, m_slope ) ) )
                    //      : 0;
                    let change : f64 =
                        if (self.counter - 1) < self.note_samples {
                            (self.freq_start - self.freq_end) as f64
                            * (1.0 - len.powf(self.freq_slope as f64))
                        } else {
                            0.0
                        };

                    // m_freq = m_endFreq + change;
                    self.note_freq =
                        self.freq_end as f64 + change;
                }

                offs += 1;
            }
        });
    }
}

impl MonoVoice for Op_Kickmess {
    fn start_note(&mut self, offs: usize, freq: f32, _vel: f32) {
        self.counter = 0;
        self.note_offs = Some(offs);

        self.note_samples =
            ((self.length_ms as f32 * self.srate) / 1000.0) as usize;

        if self.freq_note_start {
            self.note_freq = freq as f64;
        } else {
            self.note_freq = self.freq_start as f64;
        }

        self.cur_phase = self.phase;
    }

    fn end_note(&mut self, offs: usize) {
    }

    fn is_playing(&self) -> bool { self.counter > 0 }
}

