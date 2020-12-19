pub trait Channel {
    fn process(&mut self, f: &mut dyn FnMut(&[f32], &mut [f32]));
}


#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Param {
    None,
    Freq1,
    Freq2,
    Decay1,
    Decay2,
    Release1,
    Release2,
    Gain1,
    Gain2,
    Dist1,
    Dist2,
    Noise1,
    Env1,
    Phase1,
    S1,
    S2,
    Threshold1,
    Threshold2,
    Tmp1,
    Tmp2,
}

pub trait ParamMapper {
    fn map(&self, p: Param) -> Param;
}

impl ParamMapper for dyn Fn(Param) -> Param {
    fn map(&self, p: Param) -> Param { self(p) }
}

impl ParamMapper for (Param, Param) {
    fn map(&self, p: Param) -> Param {
        if self.0 == p { self.1 }
        else           { p }
    }
}

pub trait ParamProvider {
    fn param(&self, p: Param) -> f32;
}

impl ParamProvider for (&dyn ParamProvider, &dyn ParamMapper) {
    fn param(&self, p: Param) -> f32 {
        self.0.param(self.1.map(p))
    }
}

impl ParamProvider for f32 {
    fn param(&self, _p: Param) -> f32 { *self }
}

#[derive(Debug, Clone, Copy)]
pub struct ParamDefinition(Param, f32, f32, f32, &'static str, bool);

impl ParamDefinition {
    pub fn from(p: Param, min: f32, max: f32, def: f32, desc: &'static str) -> Self {
        Self(p, min, max, def, desc, false)
    }

    pub fn exp(mut self) -> Self {
        self.5 = true;
        self
    }

    pub fn param(&self) -> Param { self.0 }

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
        self.defines.push(pd)
    }

    pub fn add2(&mut self, other: &mut Self, pd: ParamDefinition) {
        other.add(pd);
        self.defines.push(pd)
    }

    pub fn param_count(&self) -> usize { self.defines.len() }
    pub fn definition(&self, idx: usize) -> Option<&ParamDefinition> {
        if idx >= self.defines.len() {
            return None;
        }

        Some(&self.defines[idx])
    }

    pub fn idx_of(&self, p: Param) -> Option<usize> {
        for (i, d) in self.defines.iter().enumerate() {
            if d.0 == p {
                return Some(i);
            }
        }

        None
    }

    pub fn get_raw(&self, idx: usize, pp: &dyn ParamProvider) -> f32 {
        if let Some(pd) = self.definition(idx) {
            pp.param(pd.0)
        } else {
            0.0
        }
    }

    pub fn get(&self, idx: usize, pp: &dyn ParamProvider) -> f32 {
        if let Some(pd) = self.definition(idx) {
            pd.map(pp.param(pd.0))
        } else {
            0.0
        }
    }
}

// Foldback:
//    if( in >= m_threshold || in < -m_threshold )
//    {
//        return ( fabsf( fabsf( fmodf( in - m_threshold, m_threshold*4 ) ) - m_threshold*2 ) - m_threshold ) * m_gain;
//    }
//    return in * m_gain;

pub trait MonoProcessor {
    fn init_params(ps: &mut ParamSet, public_ps: &mut ParamSet);
    fn read_params(&mut self, ps: &ParamSet, pp: &dyn ParamProvider);
    fn process(&mut self, c: &mut dyn Channel);
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
    last_frame_nf:  usize,
}

impl SmoothParameters {
    pub fn new(framesize: usize, param_count: usize) -> Self {
        let mut v1 = Vec::with_capacity(framesize * param_count);
        let mut v2 = Vec::with_capacity(framesize * param_count);
        v1.resize(framesize * param_count, 0.0);
        v2.resize(framesize * param_count, 0.0);

        Self {
            current:       v1,
            last:          v2,
            last_frame_nf: 0,
            framesize,
            param_count,
        }
    }

    /// Initialize the current frame values, in case we never ran a single frame.
    pub fn init_params(&mut self, frames: usize, ps: &ParamSet, pp: &dyn ParamProvider) {
        let v         = &mut self.current;
        let framesize = self.framesize;

        for pi in 0..self.param_count {
            let pv = ps.get(pi, pp);

            for i in 0..framesize {
                v[i * framesize + pi] = pv;
            }
        }
    }

    pub fn advance_params(&mut self, frames: usize,
                          end_param_frame: usize,
                          ps: &ParamSet,
                          pp: &dyn ParamProvider) {

        let v      = &mut self.current;
        let last_v = &self.last[(self.last.len() - frames)..self.last.len()];

        let last_frame_frame = self.last_frame_nf + self.framesize;

        for pi in 0..self.param_count {
            let end_param_val = ps.get(pi, pp) as f64;
            let last_val      = last_v[pi] as f64;

            for i in 0..frames {
                // calculate the interpolation factor between last_v and
                // the current frame number:
                let x : f64 =
                    (i + last_frame_frame) as f64 / end_param_frame as f64;

                v[i * frames + pi] =
                    (last_val * (1.0 - x) + x * end_param_val) as f32;
            }
        }

        self.last_frame_nf = last_frame_frame + frames;
    }

    pub fn get_frame(&self, idx: usize) -> &[f32] {
        let frame_idx = idx * self.framesize;
        &self.current[frame_idx..(frame_idx + self.framesize)]
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.last, &mut self.current);
    }
}
