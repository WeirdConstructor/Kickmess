// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

#![allow(warnings)]
pub mod proc;
pub mod helpers;
mod op_kickmess;
mod env;
pub mod editor;
pub mod ui;
pub mod window;

use proc::{ParamProvider, VoiceManager, ParamDefinition, ParamSet, MonoProcessor, MonoVoice, SmoothParameters};
use op_kickmess::*;
use helpers::note_to_freq;

#[macro_use]
extern crate vst;

use vst::util::AtomicFloat;
use vst::api::Events;
use vst::event::{Event, MidiEvent};
use vst::buffer::AudioBuffer;
use vst::plugin::{HostCallback, Category, Info, Plugin, PluginParameters, CanDo};

use std::sync::Arc;

const MAX_BLOCKSIZE: usize = 64;
const MAX_POLY:      usize = 16;

struct Kickmess {
    host:           HostCallback,
    params:         Arc<KickmessVSTParams>,
    voices:         VoiceManager<OpKickmess>,
    smooth_param:   SmoothParameters,
}

impl Default for Kickmess {
    fn default() -> Kickmess {
        Kickmess {
            host:   HostCallback::default(),
            params: Arc::new(KickmessVSTParams::default()),
            voices: VoiceManager::new(MAX_POLY),
            smooth_param: SmoothParameters::new(MAX_BLOCKSIZE, 0),
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
        }
    }

    fn init(&mut self) {
        helpers::init_cos_tab();
    }

    fn get_info(&self) -> Info {
        Info {
            name:         "Kickmess (VST)".to_string(),
            vendor:       "Weird Constructor".to_string(),
            inputs:       0,
            outputs:      1,
            midi_inputs:  1,
            midi_outputs: 0,
            parameters:   self.params.public_ps.param_count() as i32,
            unique_id:    934843292,
            version:      0001,
            category:     Category::Synth,
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

        for os in out_buf.iter_mut() { *os = 0.0; }

        loop {
            let advance_frames =
                if remaining > MAX_BLOCKSIZE { MAX_BLOCKSIZE } else { remaining };

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
    }

    fn process_events(&mut self, events: &Events) {
        for e in events.events() {
            match e {
                Event::Midi(MidiEvent { data, delta_frames, .. }) => {
                    self.voices.handle_midi(&data, delta_frames as usize);
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
            editor::KickmessEditor::new(self.host, self.params.clone())))
    }
}

pub(crate) struct KickmessVSTParams {
    ps:             ParamSet,
    public_ps:      ParamSet,
    params:         Vec<AtomicFloat>,
    dirty_params:   Vec<std::sync::atomic::AtomicBool>,
}

impl KickmessVSTParams {
    fn set(&self, idx: usize, val: f32) {
        if let Some(af) = self.params.get(idx) {
            af.set(val);
            self.dirty_params[idx].store(true,
                std::sync::atomic::Ordering::Relaxed);
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
        let mut dirty_params = vec![];

        for idx in 0..ps.param_count() {
            params.push(AtomicFloat::new(ps.definition(idx).unwrap().default_p()));
            dirty_params.push(std::sync::atomic::AtomicBool::new(true));
        }

        KickmessVSTParams {
            ps,
            public_ps,
            params,
            dirty_params,
        }
    }
}

impl PluginParameters for KickmessVSTParams {
    fn get_parameter(&self, index: i32) -> f32 {
        self.public_ps.get_raw(index as usize, self)
    }

    fn set_parameter(&self, index: i32, val: f32) {
        if let Some(pd) = self.public_ps.definition(index as usize) {
            self.set(pd.idx(), val);
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        if index > self.public_ps.param_count() as i32 {
            return "".to_string();
        }

        let v = self.get_parameter(index);
        let pd = self.public_ps.definition(index as usize).unwrap();
        format!("{} >= {:.2} >= {}", pd.min(), pd.map(v), pd.max())
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


//use lv2::prelude::*;
//
//#[derive(PortCollection)]
//struct Ports {
//    gain: InputPort<Control>,
//    input: InputPort<Audio>,
//    output: OutputPort<Audio>,
//}
//
//#[uri("https://github.com/WeirdConstructor/kickmess")]
//struct Kickmess;
//
//impl Plugin for Kickmess {
//    type Ports = Ports;
//
//    type InitFeatures = ();
//    type AudioFeatures = ();
//
//    fn new(_plugin_info: &PluginInfo, _features: &mut ()) -> Option<Self> {
//        Some(Self)
//    }
//
//    fn run(&mut self, ports: &mut Ports, _features: &mut (), _: u32) {
//        let coef = if *(ports.gain) > -90.0 {
//            10.0_f32.powf(*(ports.gain) * 0.05)
//        } else {
//            0.0
//        };
//
//        for (in_frame, out_frame) in Iterator::zip(ports.input.iter(), ports.output.iter_mut()) {
//            *out_frame = in_frame * coef;
//        }
//    }
//}
//
//lv2_descriptors!(Kickmess);
