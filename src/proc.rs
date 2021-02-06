// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

use crate::ui::protocol::UIValueSpec;

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
pub enum ParamRMode {
    Lin,
    Exp,
    Exp4,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamDefinition(usize, f32, f32, f32, &'static str, ParamRMode, bool, usize, usize);

impl ParamDefinition {
    pub fn new() -> Self {
        Self(0, 0.0, 0.0, 0.0, "", ParamRMode::Lin, false, 5, 2)
    }

    pub fn to_ui_value_spec(&self) -> UIValueSpec {
        match self.5 {
            ParamRMode::Lin => {
                let def = crate::helpers::range2p(self.3, self.1, self.2) as f64;
                UIValueSpec::new_min_max(self.1 as f64, self.2 as f64, self.7, self.8).default(def)
            }
            ParamRMode::Exp => {
                let def = crate::helpers::range2p_exp(self.3, self.1, self.2) as f64;
                UIValueSpec::new_min_max_exp(self.1 as f64, self.2 as f64, self.7, self.8).default(def)
            }
            ParamRMode::Exp4 => {
                let def = crate::helpers::range2p_exp4(self.3, self.1, self.2) as f64;
                UIValueSpec::new_min_max_exp4(self.1 as f64, self.2 as f64, self.7, self.8).default(def)
            }
        }
    }

    pub fn from(idx: usize, min: f32, max: f32, def: f32, width: usize, prec: usize, desc: &'static str) -> Self {
        Self(idx, min, max, def, desc, ParamRMode::Lin, true, width, prec)
    }

    pub fn lin(mut self) -> Self {
        self.5 = ParamRMode::Lin;
        self
    }

    pub fn exp(mut self) -> Self {
        self.5 = ParamRMode::Exp;
        self
    }

    pub fn exp4(mut self) -> Self {
        self.5 = ParamRMode::Exp4;
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
        match self.5 {
            ParamRMode::Lin  => crate::helpers::range2p(self.3, self.1, self.2),
            ParamRMode::Exp  => crate::helpers::range2p_exp(self.3, self.1, self.2),
            ParamRMode::Exp4 => crate::helpers::range2p_exp4(self.3, self.1, self.2),
        }
    }

    pub fn is_smooth(&self) -> bool { self.6 }

    pub fn map(&self, p: f32) -> f32 {
        match self.5 {
            ParamRMode::Lin  => crate::helpers::p2range(p, self.1, self.2),
            ParamRMode::Exp  => crate::helpers::p2range_exp(p, self.1, self.2),
            ParamRMode::Exp4 => crate::helpers::p2range_exp4(p, self.1, self.2),
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
    fn new() -> Self;
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
pub last_frame_cnt: usize,
pub last_frame_idx: usize,
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
            let end_param_val = ps.get(pi, pp) as f32;
            let last_val      = last_v[pi] as f32;
//            println!("LASTVAL[pi={}]: {:?} => target={} | f={}, tot={}",
//                     pi, last_val, end_param_val, frames, total_nframes);

            if (end_param_val - last_val).abs() > std::f32::EPSILON
                && ps.is_smooth(pi) {

                let inc_val =
                    (end_param_val - last_val)
                    / (total_nframes - last_frame_cnt) as f32;

                for i in 0..frames {
                    v[i * param_count + pi] =
                        last_val + (inc_val as f32 * (i + 1) as f32);
//                    println!(
//                        "[pi={}] i={}, lfc={}, iv={} => {}",
//                        pi, i, last_frame_cnt, inc_val,
//                        v[i * param_count + pi]);

//                    println!("[{} @ {},{}] x= {} | {} => {} :::=> {}",
//                             last_frame_cnt + i, last_frame_cnt,
//                             frames, x, last_val, end_param_val,
//                             v[i * param_count + pi]);
                }
            } else {
                for i in 0..frames {
                    v[i * param_count + pi] = end_param_val;
                }
            }
        }

        self.last_frame_cnt = self.last_frame_cnt + frames;
        if self.last_frame_cnt >= total_nframes {
            self.last_frame_cnt = 0;
        }
        self.last_frame_idx = (frames - 1) * param_count;
    }

    pub fn get_last_frame(&self) -> &[f32] {
        let frame_idx = self.last_frame_idx;
        &self.last[frame_idx..(frame_idx + self.param_count)]
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

pub enum VoiceEvent {
    Start { note: u8, vel: u8, delta_frames: usize },
    End   { note: u8, delta_frames: usize },
}

pub struct VoiceManager<T: MonoVoice> {
    voices: Vec<T>,
    events: Vec<VoiceEvent>,
}

impl<T: MonoVoice> VoiceManager<T> {
    pub fn new(max_voices: usize) -> Self {
        let mut voices = vec![];

        // Assumption: 10 * max_voices is enough :-)
        let events = std::vec::Vec::with_capacity(10 * max_voices);

        for _ in 0..max_voices {
            voices.push(T::new());
        }

        Self {
            voices,
            events,
        }
    }

    pub fn set_sample_rate(&mut self, rate: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_sample_rate(rate);
        }
    }

    pub fn handle_midi(&mut self, data: &[u8], delta_frames: usize, my_channel: u8) {
        let cmd  = (data[0] & 0xF0) >> 4;
        let chan = data[0] & 0x0F;

        println!("MIDI {} {}: {} DT: {}", chan, my_channel, cmd, delta_frames);

        if my_channel != chan {
            return;
        }

        if cmd == 0b1001 {
            //d// println!("RECV: {} DT: {}", data[0], delta_frames);
            self.events.push(VoiceEvent::Start {
                note:         data[1],
                vel:          data[2],
                delta_frames: delta_frames as usize,
            });

        } else if cmd == 0b1000 {
            self.events.push(VoiceEvent::End {
                note:         data[1],
                delta_frames: delta_frames as usize,
            });
        }
    }

    pub fn process(&mut self, nframe_offs: usize, out: &mut [f32], smooth_param: &SmoothParameters) {
        self.process_voice_events();

        for voice in self.voices.iter_mut() {
            if voice.is_playing() {
                voice.process(smooth_param, nframe_offs, out);
            }
        }
    }

    fn process_voice_events(&mut self) {
        use crate::helpers::note_to_freq;

        while !self.events.is_empty() {
            match self.events.pop().unwrap() {
                VoiceEvent::Start { note, delta_frames, vel } => {
                    for voice in self.voices.iter_mut() {
                        if !voice.is_playing() {
                            voice.start_note(
                                note as usize,
                                delta_frames as usize,
                                note_to_freq(note as f32),
                                vel as f32);
                            break;
                        }
                    }
                },
                VoiceEvent::End { note, delta_frames } => {
                    for voice in self.voices.iter_mut() {
                        if voice.id() == (note as usize) {
                            voice.end_note(delta_frames);
                            break;
                        }
                    }
                },
            }
        }
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
        ps.add(ParamDefinition::from(1,  5.0, 3000.0, 150.0, 3, 1, "Start Freq.").exp().smooth());
        ps.add(ParamDefinition::from(2,  5.0, 2000.0,  40.0, 3, 1, "End Freq.").exp().no_smooth());
        ps.add(ParamDefinition::from(3,  5.0, 5000.0, 440.0, 3, 1, "Length").exp().smooth());

        let new_params = vec![0.0, 0.3, 0.4, 0.5];
        smooth.advance_params(2, 66, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 274.55, 324.20, 1253.75]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(1)),
            "[0.00, 274.55, 324.20, 1253.75]");

        let new_params = vec![0.0, 1.0, 1.0, 1.0];
        smooth.advance_params(64, 66, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 317.14, 2000.00, 1312.29]");
//            "[0.00, 358.41, 2000.00, 1369.02]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 3000.00, 2000.00, 5000.00]");


        let new_params = vec![0.0, 0.0, 0.0, 0.0];
        smooth.advance_params(64, 64, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 2953.20, 5.00, 4921.95]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 5.00, 5.00, 5.00]");

        // new run...
        let new_params = vec![1.0, 1.0, 1.0, 1.0];
        smooth.advance_params(64, 256, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 16.70, 2000.00, 24.51]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 753.75, 2000.00, 1253.75]");

        smooth.advance_params(64, 256, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_last_frame()),
            "[0.00, 753.75, 2000.00, 1253.75]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 765.45, 2000.00, 1273.26]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 1502.50, 2000.00, 2502.50]");

        smooth.advance_params(64, 256, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 1514.20, 2000.00, 2522.01]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 2251.25, 2000.00, 3751.25]");

        smooth.advance_params(64, 256, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 2262.95, 2000.00, 3770.76]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 3000.00, 2000.00, 5000.00]");

        // run with return to original
        let new_params = vec![0.5, 0.5, 0.5, 0.5];
        smooth.advance_params(64, 256, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 2991.23, 503.75, 4985.37]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 2438.44, 503.75, 4063.44]");

        let new_params = vec![1.0, 1.0, 1.0, 1.0];
        smooth.advance_params(64, 256, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 2441.36, 2000.00, 4068.32]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 2625.62, 2000.00, 4375.62]");

        smooth.advance_params(64, 256, &ps, &new_params);
        smooth.advance_params(64, 256, &ps, &new_params);

        assert_eq!(
            &fmt_vec(&smooth.get_frame(0)),
            "[0.00, 2815.74, 2000.00, 4692.69]");
        assert_eq!(
            &fmt_vec(&smooth.get_frame(63)),
            "[0.00, 3000.00, 2000.00, 5000.00]");
    }
}
