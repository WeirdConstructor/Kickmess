// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

pub trait ParamProvider {
    fn param(&self, p: usize) -> f32;
}

impl ParamProvider for std::vec::Vec<f32> {
    fn param(&self, p: usize) -> f32 {
        if let Some(v) = self.get(p) {
            *v
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParamDefinition(usize, f32, f32, f32, &'static str, bool, bool);

impl ParamDefinition {
    pub fn new() -> Self {
        Self(0, 0.0, 0.0, 0.0, "", false, false)
    }

    pub fn from(idx: usize, min: f32, max: f32, def: f32, desc: &'static str) -> Self {
        Self(idx, min, max, def, desc, false, true)
    }

    pub fn lin(mut self) -> Self {
        self.5 = false;
        self
    }

    pub fn exp(mut self) -> Self {
        self.5 = true;
        self
    }

    pub fn smooth(mut self) -> Self {
        self.6 = true;
        self
    }

    pub fn no_smooth(mut self) -> Self {
        self.6 = false;
        self
    }

    pub fn idx(&self) -> usize { self.0 }

    pub fn name(&self) -> &'static str { self.4 }

    pub fn min(&self)       -> f32 { self.1 }
    pub fn max(&self)       -> f32 { self.2 }

    pub fn default_p(&self) -> f32 {
        if self.5 {
            crate::helpers::range2p_exp(self.3, self.1, self.2)
        } else {
            crate::helpers::range2p(self.3, self.1, self.2)
        }
    }

    pub fn is_smooth(&self) -> bool { self.6 }

    pub fn map(&self, p: f32) -> f32 {
        if self.5 {
            crate::helpers::p2range_exp(p, self.1, self.2)
        } else {
            crate::helpers::p2range(p, self.1, self.2)
        }
    }
}

pub struct ParamSet {
    defines: Vec<ParamDefinition>,
}

impl ParamSet {
    pub fn new() -> Self {
        Self { defines: vec![] }
    }

    pub fn add(&mut self, pd: ParamDefinition) {
        if pd.idx() >= self.defines.len() {
            self.defines.resize(pd.idx() + 1, ParamDefinition::new());
        }
        self.defines[pd.idx()] = pd;
    }

    pub fn param_count(&self) -> usize { self.defines.len() }
    pub fn definition(&self, idx: usize) -> Option<&ParamDefinition> {
        if idx >= self.defines.len() {
            return None;
        }

        Some(&self.defines[idx])
    }

    pub fn get_raw(&self, idx: usize, pp: &dyn ParamProvider) -> f32 {
        if let Some(pd) = self.definition(idx) {
            pp.param(pd.0)
        } else {
            0.0
        }
    }

    pub fn is_smooth(&self, idx: usize) -> bool {
        if let Some(pd) = self.definition(idx) {
            return pd.is_smooth();
        }

        false
    }

    pub fn get(&self, idx: usize, pp: &dyn ParamProvider) -> f32 {
        if let Some(pd) = self.definition(idx) {
            pd.map(pp.param(pd.0))
        } else {
            0.0
        }
    }
}

pub trait MonoProcessor {
    fn init_params(ps: &mut ParamSet, public_ps: &mut ParamSet);
    fn process(&mut self, params: &SmoothParameters, offs: usize, out: &mut [f32]);
    fn set_sample_rate(&mut self, srate: f32);
}

pub trait MonoVoice : MonoProcessor {
    fn id(&self) -> usize;
    fn start_note(&mut self, id: usize, offs: usize, freq: f32, vel: f32);
    fn end_note(&mut self, offs: usize);
    fn is_playing(&self) -> bool;
    fn in_release(&self) -> bool;
}

pub struct SmoothParameters {
    current:        Vec<f32>,
    last:           Vec<f32>,
    framesize:      usize,
    param_count:    usize,
    last_frame_cnt: usize,
    last_frame_idx: usize,
    uninitialized:  bool,
}

impl SmoothParameters {
    /// Create a SmoothParameters structure for the given
    /// maximum framesize with a certain number of parameters.
    ///
    /// The only allocation will happend in new() here.
    pub fn new(framesize: usize, param_count: usize) -> Self {
        let mut v1 = Vec::with_capacity(framesize * param_count);
        let mut v2 = Vec::with_capacity(framesize * param_count);
        v1.resize(framesize * param_count, 0.0);
        v2.resize(framesize * param_count, 0.0);

        Self {
            current:       v1,
            last:          v2,
            last_frame_cnt: 0,
            last_frame_idx: 0,
            uninitialized: true,
            framesize,
            param_count,
        }
    }

    /// Initialize the current frame values, in case we never ran a single frame.
    fn init_params(&mut self, frames: usize, ps: &ParamSet, pp: &dyn ParamProvider) {
        let v           = &mut self.current;
        let framesize   = self.framesize;
        let param_count = self.param_count;

        for pi in 0..self.param_count {
            let pv = ps.get(pi, pp);

            for i in 0..framesize {
                v[i * param_count + pi] = pv;
            }
        }

        self.last_frame_cnt = frames;
        self.last_frame_idx = (frames - 1) * param_count;
        self.uninitialized  = false;
    }

    /// Advance the parameter interpolation.
    ///
    /// - frames contains the number of frames to interpolate from the last
    ///   call to either init_params() or advance_params()
    /// - total_nframes is the number of frames the given set of parameters
    ///   provide the value for. After total_nframes the values of the
    ///   parameters need to be reached.
    pub fn advance_params(&mut self,
                          frames: usize,
                          total_nframes: usize,
                          ps: &ParamSet,
                          pp: &dyn ParamProvider) {

        if self.uninitialized {
            self.init_params(frames, ps, pp);
            return;
        }

        self.swap();

        let v                = &mut self.current;
        let param_count      = self.param_count;
        let last_frame_cnt   = self.last_frame_cnt;
        let last_frame_idx   = self.last_frame_idx;

        let last_v = &self.last[last_frame_idx..(last_frame_idx + param_count)];

        for pi in 0..param_count {
            let end_param_val = ps.get(pi, pp) as f64;
            let last_val      = last_v[pi] as f64;

            if (end_param_val - last_val).abs() > std::f64::EPSILON
               && ps.is_smooth(pi) {

                let nframe_increment = 1.0 / (total_nframes - 1) as f64;

                for i in 0..frames {
                    // calculate the interpolation factor between last_v and
                    // the current frame number:
                    let x : f64 =
                        (i + last_frame_cnt) as f64 * nframe_increment;

                    v[i * param_count + pi] =
                        (last_val * (1.0 - x) + x * end_param_val) as f32;

//                    println!("[{} @ {},{}] x= {} | {} => {} :::=> {}",
//                             last_frame_cnt + i, last_frame_cnt,
//                             frames, x, last_val, end_param_val,
//                             v[i * param_count + pi]);
                }
            } else {
                for i in 0..frames {
                    v[i * param_count + pi] = end_param_val as f32;
                }
            }
        }

        self.last_frame_cnt = self.last_frame_cnt + frames;
        if self.last_frame_cnt >= total_nframes {
            self.last_frame_cnt = 0;
        }
        self.last_frame_idx = (frames - 1) * param_count;
    }

    /// Returns a reference to the parameters of the recently
    /// init_params() or advance_params() frames.
    pub fn get_frame(&self, idx: usize) -> &[f32] {
        let frame_idx = idx * self.param_count;
        &self.current[frame_idx..(frame_idx + self.param_count)]
    }

    fn swap(&mut self) {
        std::mem::swap(&mut self.last, &mut self.current);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt_vec(v: &[f32]) -> String {
        let mut s = String::from("[");
        for i in 0..v.len() {
            if i > 0 {
                s += ", ";
            }
            s += &format!("{:.2}", v[i]);
        }
        s += "]";
        s
    }

    #[test]
    fn check_init() {
        let mut smooth = SmoothParameters::new(64, 4);

        let mut ps = ParamSet::new();
        ps.add(ParamDefinition::from(1,  5.0, 3000.0, 150.0, "Start Freq.").exp().smooth());
        ps.add(ParamDefinition::from(2,  5.0, 2000.0,  40.0, "End Freq.").exp().no_smooth());
        ps.add(ParamDefinition::from(3,  5.0, 5000.0, 440.0, "Length").exp().smooth());

        let new_params = vec![0.0, 0.3, 0.4, 0.5];
        smooth.advance_params(2, 66, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.current[0..4]),
            "[0.00, 274.55, 324.20, 1253.75]");
        assert_eq!(
            &fmt_vec(&smooth.current[4..8]),
            "[0.00, 274.55, 324.20, 1253.75]");

        let new_params = vec![0.0, 1.0, 1.0, 1.0];
        smooth.advance_params(64, 66, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.current[(63 * 4)..(64 * 4)]),
            "[0.00, 3000.00, 2000.00, 5000.00]");
        assert_eq!(
            &fmt_vec(&smooth.current[0..4]),
            "[0.00, 358.41, 2000.00, 1369.02]");


        let new_params = vec![0.0, 0.0, 0.0, 0.0];
        smooth.advance_params(64, 64, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.current[(63 * 4)..(64 * 4)]),
            "[0.00, 5.00, 5.00, 5.00]");
        assert_eq!(
            &fmt_vec(&smooth.current[0..4]),
            "[0.00, 3000.00, 5.00, 5000.00]");
    }
}
