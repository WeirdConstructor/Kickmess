// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

#![allow(warnings)]
pub mod proc;
pub mod helpers;
mod op_kickmess;
mod env;
mod ringbuf_shared;
mod param_model;
mod filter;
mod oscillator;
mod lfo;
mod log;
pub mod editor;
pub mod ui;
pub mod window;

pub use op_kickmess::OpKickmess;

use proc::{ParamProvider, VoiceManager, ParamDefinition, MonoVoice, SmoothParameters};
pub use proc::MonoProcessor;
pub use proc::ParamSet;
use op_kickmess::*;
use helpers::note_to_freq;
use log::Log;
use core::arch::x86_64::{
    _MM_FLUSH_ZERO_ON,
    _MM_SET_FLUSH_ZERO_MODE,
    _MM_GET_FLUSH_ZERO_MODE
};

#[macro_use]
extern crate vst;

pub const DEBUG_LOGGING : bool = true;

use vst::util::AtomicFloat;
use vst::api::Events;
use vst::event::{Event, MidiEvent};
use vst::buffer::AudioBuffer;
use vst::plugin::{HostCallback, Category, Info, Plugin, PluginParameters, CanDo};
use vst::host::Host;

use std::sync::Arc;

const MAX_BLOCKSIZE: usize = 64;
const MAX_POLY:      usize = 16;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Kickmess {
    host:           HostCallback,
    params:         Arc<KickmessVSTParams>,
    voices:         VoiceManager<OpKickmess>,
    smooth_param:   SmoothParameters,
    log:            Log,
}

impl Default for Kickmess {
    fn default() -> Kickmess {
        Kickmess {
            host:   HostCallback::default(),
            params: Arc::new(KickmessVSTParams::default()),
            voices: VoiceManager::new(MAX_POLY),
            smooth_param: SmoothParameters::new(MAX_BLOCKSIZE, 0),
            log:    Log::new(),
        }
    }
}

impl Plugin for Kickmess {
    fn new(host: HostCallback) -> Self {
        let params       = Arc::new(KickmessVSTParams::default());
        let smooth_param = SmoothParameters::new(MAX_BLOCKSIZE, params.ps.param_count());

        Self {
            host,
            voices: VoiceManager::new(MAX_POLY),
            params,
            smooth_param,
            log: Log::new(),
        }
    }

    fn init(&mut self) {
        helpers::init_cos_tab();
        if DEBUG_LOGGING {
            use std::io::Write;
            self.log.start_writer_thread();
        }
    }

    fn get_info(&self) -> Info {
        let name =
            if cfg!(feature="mega") { "Megamess (VST)".to_string() }
            else                    { "Kickmess (VST)".to_string() };
        let unique_id =
            if cfg!(feature="mega") { 934843291 } else { 934843292 };

        Info {
            name,
            unique_id,
            vendor:        "Weird Constructor".to_string(),
            inputs:        0,
            outputs:       1,
            midi_inputs:   1,
            midi_outputs:  0,
            parameters:    self.params.public_ps.param_count() as i32,
            version:       0221,
            category:      Category::Synth,
            preset_chunks: true,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.voices.set_sample_rate(rate);
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut outputbuf) = buffer.split();
        let out_buf       = outputbuf.get_mut(0);
        let mut remaining = out_buf.len();
        let mut offs      = 0;

        if DEBUG_LOGGING {
            crate::log::global_set_log(&mut self.log);
        }

        let prev_ftz = unsafe { _MM_GET_FLUSH_ZERO_MODE() };
        unsafe {
            _MM_SET_FLUSH_ZERO_MODE(_MM_FLUSH_ZERO_ON);
        }

        for os in out_buf.iter_mut() { *os = 0.0; }

//        let tiflag = {
//            use vst::api::*;
//            TimeInfoFlags::PPQ_POS_VALID | TimeInfoFlags::BARS_VALID | TimeInfoFlags::TEMPO_VALID | TimeInfoFlags::VST_CLOCK_VALID
//        };
//
//        if let Some(ti) = self.host.get_time_info(tiflag.bits()) {
//            use std::io::Write;
//            use vst::api::*;
//            let tif = TimeInfoFlags::from_bits_truncate(ti.flags);
//
//            if tif.contains(TimeInfoFlags::PPQ_POS_VALID) {
//                self.log.log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//                    write!(bw, "PPQ VALID");
//                });
//            }
//
//            if tif.contains(TimeInfoFlags::BARS_VALID) {
//                self.log.log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//                    write!(bw, "BARS VALID");
//                });
//            }
//
//            if tif.contains(TimeInfoFlags::TEMPO_VALID) {
//                self.log.log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//                    write!(bw, "TEMPO VALID");
//                });
//            }
//
//            if tif.contains(TimeInfoFlags::VST_CLOCK_VALID) {
//                self.log.log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//                    write!(bw, "VST CLOCK VALID");
//                });
//            }
//
//            self.log.log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//                write!(bw, "TI: spos={}, sr={}, ppq_pos={}, tempo={}, bar_start_pos={}, samples_to_next_clock={}",
//                    ti.sample_pos,
//                    ti.sample_rate,
//                    ti.ppq_pos,
//                    ti.tempo,
//                    ti.bar_start_pos,
//                    ti.samples_to_next_clock);
//            });
//        }

        loop {
            let advance_frames =
                if remaining > MAX_BLOCKSIZE { MAX_BLOCKSIZE } else { remaining };

            let (lc, li) = (self.smooth_param.last_frame_cnt, self.smooth_param.last_frame_idx);

//            self.log.log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//                use std::io::Write;
//                write!(bw, "adv: [{:4}] {:4} => {:4}, 3 in: {}",
//                       lc, advance_frames, out_buf.len(),
//                       self.params.param(3)).unwrap();
//            });

            self.smooth_param.advance_params(
                advance_frames, out_buf.len(), &self.params.ps, &*self.params);

            self.voices.process(
                offs,
                &mut out_buf[offs..(offs + advance_frames)],
                &self.smooth_param);

            offs      += advance_frames;
            remaining -= advance_frames;
            if remaining == 0 {
                break;
            }
        }

        unsafe {
            _MM_SET_FLUSH_ZERO_MODE(prev_ftz);
        }
    }

    fn process_events(&mut self, events: &Events) {
        for e in events.events() {
            match e {
                Event::Midi(MidiEvent { data, delta_frames, .. }) => {
                    let my_channel =
                        self.params.ps.get(
                            crate::param_model::pid::midi_chan,
                            &*self.params);
                    self.voices.handle_midi(&data, delta_frames as usize, my_channel.floor() as u8);
                },
                _ => (),
            }
        }
    }

    fn can_do(&self, can_do: CanDo) -> vst::api::Supported {
        use vst::api::Supported::*;
        use vst::plugin::CanDo::*;

        match can_do {
            SendEvents | SendMidiEvent | ReceiveEvents | ReceiveMidiEvent => Yes,
            _ => No,
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }

    fn get_editor(&mut self) -> Option<Box<dyn vst::editor::Editor>> {
        Some(Box::new(
            editor::KickmessEditor::new(
                self.host,
                self.params.clone(),
                self.log.new_handle())))
    }
}

pub(crate) struct KickmessVSTParams {
    ps:             ParamSet,
    public_ps:      ParamSet,
    params:         Vec<AtomicFloat>,
    dirty_params:   ringbuf_shared::RingBuf<usize>,
}

impl KickmessVSTParams {
    fn set(&self, idx: usize, val: f32) {
        if let Some(af) = self.params.get(idx) {
            af.set(val);
            self.dirty_params.push(idx);
        }
    }
}

impl ParamProvider for KickmessVSTParams {
    fn param(&self, idx: usize) -> f32 {
        if let Some(af) = self.params.get(idx) {
            af.get()
        } else {
            0.0
        }
    }
}

fn new_default_atom(ps: &mut ParamSet, p: &ParamDefinition) -> AtomicFloat {
    AtomicFloat::new(ps.definition(p.idx()).unwrap().default_p())
}

impl Default for KickmessVSTParams {
    fn default() -> KickmessVSTParams {
        let mut ps        = ParamSet::new();
        let mut public_ps = ParamSet::new();
        OpKickmess::init_params(&mut ps, &mut public_ps);

        let mut params = vec![];

        // 10 times the parameter count, just to make sure it fits if the
        // DAW is sending too many updates per frame. Or the GUI thread is too
        // slow.
        let buf =
            crate::ringbuf_shared::RingBuf::<usize>::new(
                public_ps.param_count() * 10);

        for idx in 0..ps.param_count() {
            params.push(AtomicFloat::new(ps.definition(idx).unwrap().default_p()));
        }

        KickmessVSTParams {
            ps,
            public_ps,
            params,
            dirty_params: buf,
        }
    }
}

impl PluginParameters for KickmessVSTParams {
    fn get_parameter(&self, index: i32) -> f32 {
        self.public_ps.get_raw(index as usize, self)
    }

    fn set_parameter(&self, index: i32, val: f32) {
        if let Some(pd) = self.ps.definition(index as usize) {
            self.set(pd.idx(), val);
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        if index > self.public_ps.param_count() as i32 {
            return "".to_string();
        }

        let v = self.get_parameter(index);
        let pd = self.public_ps.definition(index as usize).unwrap();
        format!("{} <= {:.2} <= {}", pd.min(), pd.map(v), pd.max())
    }

    fn get_bank_data(&self) -> Vec<u8> {
        crate::param_model::serialize_preset(self)
    }

    fn load_bank_data(&self, data: &[u8]) {
        crate::param_model::deserialize_preset(
            data, |idx, v| {
                //d// println!("SET PARA {} = {}", idx, v);
                dbg!("load data:", idx, v);
                self.set_parameter(idx as i32, v)
            });
    }

    fn get_parameter_name(&self, index: i32) -> String {
        if let Some(pd) = self.public_ps.definition(index as usize) {
            pd.name().to_string()
        } else {
            "".to_string()
        }
    }
}

plugin_main!(Kickmess);
