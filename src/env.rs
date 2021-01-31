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

mod dahdsr {
    // Values in ms and sustain 0.0-1.0
    pub trait EnvParams {
        fn delay(&self)     -> f32;
        fn attack(&self)    -> f32;
        fn hold(&self)      -> f32;
        fn decay(&self)     -> f32;
        fn sustain(&self)   -> f32;
        fn release(&self)   -> f32;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Env {
        sample_phase:   usize,
        srate_d1k:      f32,

        /// Stores, whether we already signalled that the loop just started.
        is_start:       bool,
        last_value:     f32,
        release_value:  f32,

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
        Delay(usize),
        Attack(usize),
        Hold(usize),
        Decay(usize),
        Sustain,
        ReleaseOnOffs(usize),
        Release(usize),
        End,
    }

    impl Env {
        pub fn new() -> Self {
            Self {
                sample_phase:   0,
                srate_d1k:      0.0,
                is_start:       false,
                last_value:     0.0,
                release_value:  0.0,
                state:          EnvState::Wait,
            }
        }

        pub fn set_sample_rate(&mut self, sr: f32) {
            self.srate_d1k = sr / 1000.0;
        }

        pub fn next<P: EnvParams>(&mut self, offs: usize, p: &P) -> EnvPos {
            match self.state {
                EnvState::Wait => { return EnvPos::Off; }
                EnvState::End  => { return EnvPos::End; }
                EnvState::StartOnOffs(s_offs) => {
                    if s_offs != offs { return EnvPos::Off; }

                    self.state =
                        if p.delay() > std::f32::EPSILON {
                            EnvState::Delay(
                                (p.delay() * self.srate_d1k) as usize)

                        } else if p.attack() > std::f32::EPSILON {
                            EnvState::Attack(
                                (p.attack() * self.srate_d1k) as usize)

                        } else if p.hold() > std::f32::EPSILON {
                            EnvState::Hold(
                                (p.hold() * self.srate_d1k) as usize)

                        } else if p.decay() > std::f32::EPSILON {
                            EnvState::Decay(
                                (p.decay() * self.srate_d1k) as usize)

                        } else {
                            EnvState::Sustain
                        };

                    self.sample_phase  = 0;
                    self.last_value    = 0.0;
                    self.release_value = 0.0;
                    self.is_start      = true;
                },
                EnvState::ReleaseOnOffs(s_offs) => {
                    if s_offs != offs { return EnvPos::Off; }

                    self.state =
                        EnvState::Release((p.release() * self.srate_d1k) as usize);

                    self.release_value = self.last_value;
                    self.sample_phase = 0;
                    self.is_start = false;
                },
                _ => {},
            }

            let value =
                match self.state {
                    EnvState::Delay(delay_samples) => {
                        if self.sample_phase == delay_samples {
                            self.state =
                                if p.attack() > std::f32::EPSILON {
                                    EnvState::Attack(
                                        (p.attack() * self.srate_d1k) as usize)

                                } else if p.hold() > std::f32::EPSILON {
                                    EnvState::Hold(
                                        (p.hold() * self.srate_d1k) as usize)

                                } else if p.decay() > std::f32::EPSILON {
                                    EnvState::Decay(
                                        (p.decay() * self.srate_d1k) as usize)

                                } else {
                                    EnvState::Sustain
                                };

                            self.sample_phase = 0;

                            0.0
                        } else {
                            self.sample_phase += 1;

                            0.0
                        }
                    },
                    EnvState::Attack(attack_samples) => {
                        if self.sample_phase == attack_samples {
                            self.state =
                                if p.hold() > std::f32::EPSILON {
                                    EnvState::Hold(
                                        (p.hold() * self.srate_d1k) as usize)

                                } else if p.decay() > std::f32::EPSILON {
                                    EnvState::Decay(
                                        (p.decay() * self.srate_d1k) as usize)

                                } else {
                                    EnvState::Sustain
                                };

                            self.sample_phase = 0;

                            1.0
                        } else {
                            self.sample_phase += 1;

                            attack_samples as f32 / self.sample_phase as f32
                        }
                    },
                    EnvState::Hold(hold_samples) => {
                        if self.sample_phase == hold_samples {
                            self.state =
                                if p.decay() > std::f32::EPSILON {
                                    EnvState::Decay(
                                        (p.decay() * self.srate_d1k) as usize)

                                } else {
                                    EnvState::Sustain
                                };

                            self.sample_phase = 0;

                            1.0
                        } else {
                            self.sample_phase += 1;

                            1.0
                        }
                    },
                    EnvState::Decay(decay_samples) => {
                        if self.sample_phase == decay_samples {
                            self.state = EnvState::Sustain;

                            self.sample_phase = 0;

                            p.sustain()
                        } else {
                            self.sample_phase += 1;

                            let x = decay_samples as f32 / self.sample_phase as f32;
                            ((1.0 - x) + x * p.sustain()) as f32
                        }
                    },
                    EnvState::Sustain => {
                        p.sustain()
                    },
                    EnvState::Release(release_samples) => {
                        if self.sample_phase == release_samples {
                            self.state = EnvState::End;

                            0.0
                        } else {
                            self.sample_phase += 1;

                            let x = release_samples as f32 / self.sample_phase as f32;
                            (1.0 - x) * self.release_value
                        }
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
                | EnvState::Delay(_)
                | EnvState::Attack(_)
                | EnvState::Hold(_)
                | EnvState::Decay(_)
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
                _                 => true,
            }
        }
    }

}
