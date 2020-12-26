// Copyright (c) 2020-2021 Weird Constructor <weirdconstructor@gmail.com>
// This is a part of Kickmess. See README.md and COPYING for details.

#![allow(warnings)]
pub mod proc;
pub mod helpers;
mod op_kickmess;
mod env;
mod editor;
pub mod ui;
pub mod window;

use proc::{ParamProvider, ParamDefinition, ParamSet, MonoProcessor, MonoVoice, SmoothParameters};
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

const MAX_BLOCKSIZE: usize = 128;


impl Default for Kickmess {
    fn default() -> Kickmess {
        Kickmess {
            host:   HostCallback::default(),
            params: Arc::new(KickmessVSTParams::default()),
            voices: vec![],
            events: vec![],
            smooth_param: SmoothParameters::new(64, 0),
        }
    }
}

enum VoiceEvent {
    Start { note: u8, vel: u8, delta_frames: usize },
    End   { note: u8, delta_frames: usize },
}

struct Kickmess {
    host:           HostCallback,
    params:         Arc<KickmessVSTParams>,
    voices:         Vec<OpKickmess>,
    events:         Vec<VoiceEvent>,
    smooth_param:   SmoothParameters,
}

impl Kickmess {
    fn process_voice_events(&mut self) {
        while !self.events.is_empty() {
            match self.events.pop().unwrap() {
                VoiceEvent::Start { note, delta_frames, vel } => {
                    for voice in self.voices.iter_mut() {
                        if !voice.is_playing() {
//                            voice.read_params(&self.params.ps, &*self.params);
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

impl Plugin for Kickmess {
    fn new(host: HostCallback) -> Self {
        let max_poly = 10;
        let mut voices = vec![];
        for _ in 0..max_poly {
            voices.push(OpKickmess::new());
        }

        let events       = std::vec::Vec::with_capacity(2 * max_poly);
        let params       = Arc::new(KickmessVSTParams::default());
        let smooth_param = SmoothParameters::new(64, params.ps.param_count());

        Self {
            host,
            voices,
            params,
            events,
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
        for voice in self.voices.iter_mut() {
            voice.set_sample_rate(rate);
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut outputbuf) = buffer.split();
        let out_buf       = outputbuf.get_mut(0);
        let mut remaining = out_buf.len();
        let mut offs      = 0;

        for os in out_buf.iter_mut() { *os = 0.0; }

        loop {
            let advance_frames = if remaining > 64 { 64 } else { remaining };
            self.smooth_param.advance_params(
                advance_frames, out_buf.len(), &self.params.ps, &*self.params);

            if !self.events.is_empty() {
                self.process_voice_events();
            }

            for voice in self.voices.iter_mut() {
                if voice.is_playing() {
                    voice.process(
                        &self.smooth_param,
                        offs,
                        &mut out_buf[offs..(offs + advance_frames)]);
                }
            }

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
                    if data[0] == 144 {
                        //d// println!("RECV: {} DT: {}", data[0], delta_frames);
                        self.events.push(VoiceEvent::Start {
                            note:         data[1],
                            vel:          data[2],
                            delta_frames: delta_frames as usize,
                        });

                    } else if data[0] == 128 {
                        self.events.push(VoiceEvent::End {
                            note:         data[1],
                            delta_frames: delta_frames as usize,
                        });
                    }
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
            editor::KickmessEditor::new(self.params.clone())))
    }
}

pub(crate) struct KickmessVSTParams {
    ps:             ParamSet,
    public_ps:      ParamSet,
    params:         Vec<AtomicFloat>,
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

        for idx in 0..ps.param_count() {
            params.push(AtomicFloat::new(ps.definition(idx).unwrap().default_p()));
        }

        KickmessVSTParams {
            ps,
            public_ps,
            params,
        }
    }
}

impl PluginParameters for KickmessVSTParams {
    fn get_parameter(&self, index: i32) -> f32 {
        self.public_ps.get_raw(index as usize, self)
    }

    fn set_parameter(&self, index: i32, val: f32) {
        if let Some(pd) = self.public_ps.definition(index as usize) {
            if let Some(af) = self.params.get(pd.idx()) {
                af.set(val);
            }
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
