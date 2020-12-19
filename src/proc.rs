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
pub struct ParamDefinition(Param, f32, f32, f32, &'static str, bool, bool);

impl ParamDefinition {
    pub fn from(p: Param, min: f32, max: f32, def: f32, desc: &'static str) -> Self {
        Self(p, min, max, def, desc, false, true)
    }

    pub fn exp(mut self) -> Self {
        self.5 = true;
        self
    }

    pub fn no_smooth(mut self) -> Self {
        self.6 = false;
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
    last_frame_cnt: usize,
    last_frame_idx: usize,
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
            framesize,
            param_count,
        }
    }

    /// Initialize the current frame values, in case we never ran a single frame.
    pub fn init_params(&mut self, frames: usize, ps: &ParamSet, pp: &dyn ParamProvider) {
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

        self.swap();

        let v                = &mut self.current;
        let param_count      = self.param_count;
        let last_frame_cnt   = self.last_frame_cnt;
        let last_frame_idx   = self.last_frame_idx;
        let nframe_increment = 1.0 / (total_nframes - 1) as f64;

        let last_v = &self.last[last_frame_idx..(last_frame_idx + param_count)];

        for pi in 0..param_count {
            let end_param_val = ps.get(pi, pp) as f64;

            if ps.is_smooth(pi) {
                let last_val      = last_v[pi] as f64;

                for i in 0..frames {
                    // calculate the interpolation factor between last_v and
                    // the current frame number:
                    let x : f64 =
                        (i + last_frame_cnt) as f64 * nframe_increment;

                    v[i * param_count + pi] =
                        (last_val * (1.0 - x) + x * end_param_val) as f32;

                    println!("[{} @ {},{}] x= {} | {} => {} :::=> {}",
                             last_frame_cnt + i, last_frame_cnt,
                             frames, x, last_val, end_param_val,
                             v[i * param_count + pi]);
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

impl ParamProvider for Vec<f32> {
    fn param(&self, p: Param) -> f32 {
        self[p as usize]
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
        ps.add(ParamDefinition::from(Param::Freq1,  5.0, 3000.0, 150.0, "Start Freq.").exp());
        ps.add(ParamDefinition::from(Param::Freq2,  5.0, 2000.0,  40.0, "End Freq.").exp().no_smooth());
        ps.add(ParamDefinition::from(Param::Decay1, 5.0, 5000.0, 440.0, "Length").exp());

        let p1 = vec![0.0, 0.3, 0.4, 0.5];
        smooth.init_params(2, &ps, &p1);

        assert_eq!(
            &fmt_vec(&smooth.current[0..4]),
            "[274.55, 324.20, 1253.75, 0.00]");
        assert_eq!(
            &fmt_vec(&smooth.current[4..8]),
            "[274.55, 324.20, 1253.75, 0.00]");

        let p2 = vec![0.0, 1.0, 1.0, 1.0];
        smooth.advance_params(64, 66, &ps, &p2);

        assert_eq!(
            &fmt_vec(&smooth.current[(63 * 4)..(64 * 4)]),
            "[3000.00, 2000.00, 5000.00, 0.00]");
        assert_eq!(
            &fmt_vec(&smooth.current[0..4]),
            "[358.41, 2000.00, 1369.02, 0.00]");


        let p2 = vec![0.0, 0.0, 0.0, 0.0];
        smooth.advance_params(64, 64, &ps, &p2);

        assert_eq!(
            &fmt_vec(&smooth.current[(63 * 4)..(64 * 4)]),
            "[5.00, 5.00, 5.00, 0.00]");
        assert_eq!(
            &fmt_vec(&smooth.current[0..4]),
            "[3000.00, 5.00, 5000.00, 0.00]");
    }
}
