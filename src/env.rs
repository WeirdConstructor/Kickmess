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
    Release(usize, f64),
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
                if s_offs == offs {
                    self.state = EnvPosState::Running;
                    self.len_samples =
                        ((self.len_ms * self.srate) / 1000.0) as usize;
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

            EnvPos::Release(pos, value)
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
