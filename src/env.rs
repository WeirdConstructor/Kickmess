// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

#[derive(Debug, Clone, Copy)]
enum EnvPosState {
    Wait,
    StartOffs(usize),
    Running,
    End,
}

#[derive(Debug, Clone, Copy)]
pub enum EnvPos {
    Off,
    Running(usize, f64),
    End,
}

#[derive(Debug, Clone, Copy)]
pub struct REnv {
    len_ms:         f32,
    len_samples:    usize,
    pos:            usize,

    srate:          f32,
    state:          EnvPosState,
}

impl REnv {
    pub fn new() -> Self {
        Self {
            len_ms:         20.0,
            len_samples:    0,
            pos:            0,
            srate:          0.0,
            state:          EnvPosState::Wait,
        }
    }

    pub fn set_release(&mut self, rt: f32) {
        self.len_ms = rt;
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.srate = sr;
    }

    pub fn next(&mut self, offs: usize) -> EnvPos {
        match self.state {
            EnvPosState::Wait => { return EnvPos::Off; }
            EnvPosState::StartOffs(s_offs) => {
                //d// println!("********* {} <=> {}", s_offs, offs);
                if s_offs == offs {
                    self.state = EnvPosState::Running;
                    self.len_samples =
                        ((self.len_ms * self.srate) / 1000.0) as usize;
                    //d// println!("TRIGGER: {} {}", self.len_samples, self.len_ms);
                    self.pos   = 0;
                } else {
                    return EnvPos::Off;
                }
            },
            EnvPosState::End => { return EnvPos::End; }
            _ => (),
        }

        let pos = self.pos;

        match self.state {
            EnvPosState::Running => {
                self.pos += 1;
            },
            _ => (),
        }

        if pos >= self.len_samples {
            self.state = EnvPosState::End;
            EnvPos::End

        } else {
            let value =
                if pos < self.len_samples {
                    (pos as f64) / (self.len_samples as f64)
                } else {
                    1.0
                };

            EnvPos::Running(pos, value)
        }
    }

    pub fn reset(&mut self) {
        self.state = EnvPosState::Wait;
    }

    pub fn trigger(&mut self, offs: usize) {
        self.state = EnvPosState::StartOffs(offs);
    }

    pub fn active(&self) -> bool {
        match self.state {
            EnvPosState::StartOffs(_) => true,
            EnvPosState::Running      => true,
            _ => false,
        }
    }
}

pub mod generic {

    pub const MAX_STAGES : usize = 5;

    // Values in ms and sustain 0.0-1.0
    pub trait EnvParams {
        fn start(&self)            -> f32;
        fn pre(&self, idx: usize)  -> (f32, f32);
        fn sustain(&self)          -> f32;
        fn post(&self, idx: usize) -> (f32, f32);
    }

    impl EnvParams for (f32, (f32, f32), f32) {
        fn start(&self) -> f32 { self.0 }
        fn pre(&self, idx: usize) -> (f32, f32) {
            if idx == 0 {
                self.1
            } else {
                (-1.0, 0.0)
            }
        }

        fn sustain(&self) -> f32 { 0.0 }

        fn post(&self, idx: usize) -> (f32, f32) {
            (-1.0, 0.0)
        }
    }

    impl EnvParams for (f32, (f32, f32), (f32, f32), f32, (f32, f32)) {
        fn start(&self) -> f32 { self.0 }
        fn pre(&self, idx: usize) -> (f32, f32) {
            if idx == 0 {
                self.1
            } else if idx == 1 {
                self.2
            } else {
                (-1.0, 0.0)
            }
        }

        fn sustain(&self) -> f32 { self.3 }

        fn post(&self, idx: usize) -> (f32, f32) {
            if idx == 0 {
                self.4
            } else {
                (-1.0, 0.0)
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Env {
        /// The current stage phase, from 0.0 to 1.0
        phase:   f32,
        /// Samples per millisecond
        srate_ms:       f32,

        /// Stores, whether we already signalled that the loop just started.
        is_start:       bool,
        /// The most recently output value
        last_value:     f32,
        /// The start value of the current phase
        phase_value:    f32,

        state:          EnvState,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum EnvPos {
        Off,
        /// Running state, true value means, the env just started (reset stuff)
        /// and the float value is the envelope value.
        Running(bool, f32),
        End,
    }

    #[derive(Debug, Clone, Copy)]
    enum EnvState {
        Wait,
        StartOnOffs(usize),
        Stage { inc: f32, value: f32, idx: usize, pre: bool },
        Sustain,
        ReleaseOnOffs(usize),
        End,
    }

    impl Env {
        pub fn new() -> Self {
            Self {
                phase:          0.0,
                srate_ms:       0.0,
                is_start:       false,
                last_value:     0.0,
                phase_value:    0.0,
                state:          EnvState::Wait,
            }
        }

        pub fn set_sample_rate(&mut self, sr: f32) {
            self.srate_ms = sr / 1000.0;
        }

        #[inline]
        pub fn next_pre<P: EnvParams>(&mut self, p: &P, start: usize) -> (f32, f32, usize) {
            for i in start..MAX_STAGES {
                let (time, dest) = p.pre(i);
                if time > std::f32::EPSILON {
                    return (time * self.srate_ms, dest, i + 1);
                }
            }

            (-1.0, 0.0, 0)
        }

        #[inline]
        pub fn next_post<P: EnvParams>(&mut self, p: &P, start: usize) -> (f32, f32, usize) {
            for i in start..MAX_STAGES {
                let (time, dest) = p.post(i);
                if time > std::f32::EPSILON {
                    return (time * self.srate_ms, dest, i + 1);
                }
            }

            (-1.0, 0.0, 0)
        }

        pub fn next<P: EnvParams>(&mut self, offs: usize, p: &P) -> EnvPos {
            match self.state {
                EnvState::Wait => { return EnvPos::Off; }
                EnvState::End  => { return EnvPos::End; }
                EnvState::StartOnOffs(s_offs) => {
                    if s_offs != offs { return EnvPos::Off; }

                    let (time, value, idx) = self.next_pre(p, 0);

                    self.state =
                        if time < 1.0 {
                            if p.sustain() > 0.0 {
                                EnvState::Sustain
                            } else {
                                EnvState::End
                            }
                        } else {
                            EnvState::Stage {
                                inc: 1.0 / time,
                                value,
                                idx,
                                pre: true,
                            }
                        };
                    println!("start {:?} {} {}", self.state, time, self.srate_ms);

                    self.phase         = 0.0;
                    self.last_value    = 0.0;
                    self.phase_value   = p.start();
                    self.is_start      = true;
                },
                EnvState::ReleaseOnOffs(s_offs) => {
                    if s_offs != offs { return EnvPos::Off; }

                    let (time, value, idx) = self.next_post(p, 0);

                    self.state =
                        if time < 1.0 {
                            EnvState::End
                        } else {
                            EnvState::Stage {
                                inc: 1.0 / time,
                                value,
                                idx,
                                pre: false
                            }
                        };
                    println!("release");

                    self.phase        = 0.0;
                    self.phase_value  = self.last_value;
                    self.is_start     = false;
                },
                _ => {},
            }

            let value =
                match self.state {
                    EnvState::Stage { inc, value, idx, pre } => {
                        let x     = self.phase;
                        let value = self.phase_value * (1.0 - x) + x * value;
                        self.phase += inc;

                        if self.phase > 1.0 {
                            println!("phase reached");

                            // TODO: In the "pre phase" interpret
                            //       new values as "starting" values
                            //       for the next non zero time phase.
                            //     - In the "post phase" interpret the
                            //       next_value as the value to be reached
                            //       after the current timeout.
                            let (time, next_value, next_idx) =
                                if pre { self.next_pre(p, idx) }
                                else   { self.next_post(p, idx) };

                            self.state =
                                if time < 1.0 {

                                    self.phase = 0.0;
                                    if pre && p.sustain() > 0.0 {
                                        EnvState::Sustain
                                    } else {
                                        EnvState::End
                                    }

                                } else {
                                    let inc = 1.0 / time;

                                    self.phase_value = value;
                                    self.phase       = inc + inc;

                                    let value =
                                        self.phase_value * (1.0 - inc)
                                        + inc * next_value;

                                    EnvState::Stage {
                                        inc,
                                        value:  next_value,
                                        idx:    next_idx,
                                        pre:    pre,
                                    }
                                };
                        }

                        value
                    },
                    EnvState::Sustain => {
                        p.sustain()
                    },
                    _ => {
                        return EnvPos::End;
                    },
                };

            let was_start   = self.is_start;

            self.is_start   = false;
            self.last_value = value;

            EnvPos::Running(was_start, value)
        }

        pub fn reset(&mut self) {
            self.state = EnvState::Wait;
        }

        pub fn trigger(&mut self, offs: usize) {
            self.state = EnvState::StartOnOffs(offs);
        }

        pub fn release(&mut self, offs: usize) {
            match self.state {
                  EnvState::StartOnOffs(_)
                | EnvState::Stage { .. }
                | EnvState::Sustain => {
                    self.state = EnvState::ReleaseOnOffs(offs)
                },
                _ => { }
            }
        }

        pub fn active(&self) -> bool {
            match self.state {
                EnvState::Wait => false,
                EnvState::End  => false,
                _              => true,
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_float_tpl_eq {
        ($a:expr, $b:expr) => {
            if ($a.0 - $b.0).abs() > 0.0001 {
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#, $a, $b)
            }

            if ($a.1 - $b.1).abs() > 0.0001 {
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#, $a, $b)
            }
        }
    }


    macro_rules! assert_float_eq {
        ($a:expr, $b:expr) => {
            if ($a - $b).abs() > 0.0001 {
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#, $a, $b)
            }
        }
    }

    fn debug_print_float_vec(v: &[(f32, f32)]) {
        let per_row = 5;
        let mut row_cnt = 0;
        for i in 0..v.len() {
            eprint!("[{:4.2} {:6.4}] ", v[i].0, v[i].1);

            row_cnt += 1;
            if row_cnt >= per_row {
                eprintln!("");
                row_cnt = 0;
            }
        }
        eprintln!("");
    }

    fn gen_env_samples<P: generic::EnvParams>(env: &mut generic::Env, env_par: &P, srate: f32, samples: usize, mut release: usize) -> Vec<(f32, f32)> {
        use super::generic::*;

        env.set_sample_rate(srate);
        env.trigger(0);

        let mut out = vec![];

        for x in 0..(samples + 1) {
            let x = x as f32 / (samples as f32);

            if release == 0 { env.release(0); }
            release -= 1;

            match env.next(0, env_par) {
                EnvPos::Running(_, v) => {
                    out.push((x, v));
                },
                _ => {},
            }
        }

        out
    }

    #[test]
    fn check_adsr_short() {
        use super::generic::*;

        let mut env = Env::new();
        let env_par =
            (0.0, (0.1, 1.0), (0.14142135, 0.75), 0.75, (0.1, 0.0));

        let out = gen_env_samples(&mut env, &env_par, 80.0, 100, 69);
        //d// debug_print_float_vec(&out[..]);

        assert_eq!(out.len(), 69);

        for i in 0..out.len() {
            assert_float_eq!(out[i].1, 0.75);
        }
    }


    #[test]
    fn check_adsr() {
        use super::generic::*;

        let mut env = Env::new();
        let env_par =
            (0.0, (50.0, 1.0), (100.0, 0.75), 0.75, (50.0, 0.0));

        let out = gen_env_samples(&mut env, &env_par, 100.0, 100, 69);
        debug_print_float_vec(&out[..]);

        assert_eq!(out.len(), 75);

        // Attack 50ms
        assert_float_tpl_eq!(out[0], (0.00, 0.0));
        assert_float_tpl_eq!(out[1], (0.01, 0.2));
        assert_float_tpl_eq!(out[2], (0.02, 0.4));
        assert_float_tpl_eq!(out[3], (0.03, 0.6));
        assert_float_tpl_eq!(out[4], (0.04, 0.8));

        // Decay 100ms
        assert_float_tpl_eq!(out[5],  (0.05, 1.0));
        assert_float_tpl_eq!(out[6],  (0.06, 0.950));
        assert_float_tpl_eq!(out[7],  (0.07, 0.925));
        assert_float_tpl_eq!(out[8],  (0.08, 0.900));
        assert_float_tpl_eq!(out[9],  (0.09, 0.875));

        assert_float_tpl_eq!(out[10], (0.10, 0.850));
        assert_float_tpl_eq!(out[11], (0.11, 0.825));
        assert_float_tpl_eq!(out[12], (0.12, 0.800));
        assert_float_tpl_eq!(out[13], (0.13, 0.775));
        assert_float_tpl_eq!(out[14], (0.14, 0.750));

        // Sustain
        for i in 14..70 {
            assert_float_eq!(out[i].1, 0.75);
        }

        // Release 50 ms
        assert_float_tpl_eq!(out[70], (0.70, 0.60));
        assert_float_tpl_eq!(out[71], (0.71, 0.45));
        assert_float_tpl_eq!(out[72], (0.72, 0.30));
        assert_float_tpl_eq!(out[73], (0.73, 0.15));
        assert_float_tpl_eq!(out[74], (0.74, 0.00));
    }

    #[test]
    fn check_adr() {
        use super::generic::*;

        let mut env = Env::new();
        let env_par =
            (0.0, (50.0, 1.0), (100.0, 0.75), 0.75, (50.0, 0.0));

        let out = gen_env_samples(&mut env, &env_par, 100.0, 100, 15);
        debug_print_float_vec(&out[..]);

        assert_eq!(out.len(), 21);

        // Attack 50ms
        assert_float_tpl_eq!(out[0], (0.00, 0.0));
        assert_float_tpl_eq!(out[1], (0.01, 0.2));
        assert_float_tpl_eq!(out[2], (0.02, 0.4));
        assert_float_tpl_eq!(out[3], (0.03, 0.6));
        assert_float_tpl_eq!(out[4], (0.04, 0.8));

        // Decay 100ms
        assert_float_tpl_eq!(out[5],  (0.05, 1.0));
        assert_float_tpl_eq!(out[6],  (0.06, 0.950));
        assert_float_tpl_eq!(out[7],  (0.07, 0.925));
        assert_float_tpl_eq!(out[8],  (0.08, 0.900));
        assert_float_tpl_eq!(out[9],  (0.09, 0.875));

        assert_float_tpl_eq!(out[10], (0.10, 0.850));
        assert_float_tpl_eq!(out[11], (0.11, 0.825));
        assert_float_tpl_eq!(out[12], (0.12, 0.800));
        assert_float_tpl_eq!(out[13], (0.13, 0.775));
        assert_float_tpl_eq!(out[14], (0.14, 0.750));

        // Release 50 ms
        assert_float_tpl_eq!(out[15], (0.15, 0.75));
        assert_float_tpl_eq!(out[16], (0.16, 0.60));
        assert_float_tpl_eq!(out[17], (0.17, 0.45));
        assert_float_tpl_eq!(out[18], (0.18, 0.30));
        assert_float_tpl_eq!(out[19], (0.19, 0.15));
        assert_float_tpl_eq!(out[20], (0.20, 0.00));
    }

    #[test]
    fn check_ad() {
        use super::generic::*;

        let mut env = Env::new();
        let env_par =
            (0.0, (50.0, 1.0), (100.0, 0.0), 0.0, (50.0, 0.0));

        let out = gen_env_samples(&mut env, &env_par, 100.0, 100, 30);
        debug_print_float_vec(&out[..]);

        assert_eq!(out.len(), 14);

        // Attack 50ms
        assert_float_tpl_eq!(out[0], (0.00, 0.0));
        assert_float_tpl_eq!(out[1], (0.01, 0.2));
        assert_float_tpl_eq!(out[2], (0.02, 0.4));
        assert_float_tpl_eq!(out[3], (0.03, 0.6));
        assert_float_tpl_eq!(out[4], (0.04, 0.8));

        // Decay 100ms, last sample is implicit
        assert_float_tpl_eq!(out[5],  (0.05, 1.0));
        assert_float_tpl_eq!(out[6],  (0.06, 0.8));
        assert_float_tpl_eq!(out[7],  (0.07, 0.7));
        assert_float_tpl_eq!(out[8],  (0.08, 0.6));
        assert_float_tpl_eq!(out[9],  (0.09, 0.5));

        assert_float_tpl_eq!(out[10], (0.10, 0.4));
        assert_float_tpl_eq!(out[11], (0.11, 0.3));
        assert_float_tpl_eq!(out[12], (0.12, 0.2));
        assert_float_tpl_eq!(out[13], (0.13, 0.1));
        // assert_float_tpl_eq!(out[14], (0.14, 0.0));
    }


    #[test]
    fn check_s() {
        use super::generic::*;

        let mut env = Env::new();
        let env_par =
            (0.0, (0.0, 1.0), (0.0, 0.0), 1.0, (0.0, 0.0));

        let out = gen_env_samples(&mut env, &env_par, 100.0, 100, 10);
        debug_print_float_vec(&out[..]);

        assert_eq!(out.len(), 10);

        // Sustain
        for i in 0..10 {
            assert_float_eq!(out[i].1, 1.0);
        }

    }

    #[test]
    fn check_0() {
        use super::generic::*;

        let mut env = Env::new();
        let env_par =
            (0.0, (0.0, 1.0), (0.0, 0.0), 0.0, (0.0, 0.0));

        let out = gen_env_samples(&mut env, &env_par, 100.0, 100, 30);
        debug_print_float_vec(&out[..]);

        assert_eq!(out.len(), 0);
    }
}
