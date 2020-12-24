#![allow(warnings)]
pub mod proc;
pub mod helpers;
mod op_kickmess;
mod env;
mod editor;
//pub mod pugl;
pub mod ui;
//pub mod baseview;
pub mod baseview_femtovg;

use proc::Channel;
use proc::{ParamProvider, Param, ParamSet, MonoProcessor, MonoVoice, SmoothParameters};
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

struct AB<'a>((&'a [f32], &'a mut [f32]));


impl<'a> AB<'a> {
    fn from_buffers(bufs: (&'a [f32], &'a mut [f32])) -> Self {
        Self(bufs)
    }
}

impl<'a> Channel for AB<'a> {
    fn process(&mut self, f: &mut dyn FnMut(&[f32], &mut [f32])) {
        f(self.0.0, self.0.1)
    }
}


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
                            voice.read_params(&self.params.ps, &*self.params);
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

        let inbuf : [f32; 0] = [];

        let (_, mut outputbuf) = buffer.split();

        for os in outputbuf.get_mut(0) { *os = 0.0; }

        if !self.events.is_empty() {
            self.process_voice_events();
        }

        self.smooth_param.advance_params(
            64, outputbuf.get_mut(0).len(), &self.params.ps, &*self.params);

        let mut channel = AB::from_buffers((&inbuf, outputbuf.get_mut(0)));

        for voice in self.voices.iter_mut() {
            if voice.is_playing() {
                voice.read_params(&self.params.ps, &*self.params);
                voice.process(&mut channel);
            }
        }
    }

    fn process_events(&mut self, events: &Events) {
        for e in events.events() {
            match e {
                Event::Midi(MidiEvent { data, delta_frames, .. }) => {
                    if data[0] == 144 {
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

    freq_start:      AtomicFloat,
    freq_end:        AtomicFloat,
    length:          AtomicFloat,
    dist_start:      AtomicFloat,
    dist_end:        AtomicFloat,
    dist_gain:       AtomicFloat,
    env_slope:       AtomicFloat,
    noise:           AtomicFloat,
    freq_slope:      AtomicFloat,
    env_release:     AtomicFloat,
    freq_note_start: AtomicFloat,
    freq_note_end:   AtomicFloat,
    phase_offs:      AtomicFloat,
}

impl KickmessVSTParams {
    fn atom_float(&self, p: Param) -> Option<&AtomicFloat> {
        match p {
            Param::Freq1    => Some(&self.freq_start),
            Param::Freq2    => Some(&self.freq_end),
            Param::Decay1   => Some(&self.length),
            Param::Dist1    => Some(&self.dist_start),
            Param::Dist2    => Some(&self.dist_end),
            Param::Gain1    => Some(&self.dist_gain),
            Param::Env1     => Some(&self.env_slope),
            Param::Release1 => Some(&self.freq_slope),
            Param::Release2 => Some(&self.env_release),
            Param::Noise1   => Some(&self.noise),
            Param::S1       => Some(&self.freq_note_start),
            Param::S2       => Some(&self.freq_note_end),
            Param::Phase1   => Some(&self.phase_offs),
            _               => None,
        }
    }
}

impl ParamProvider for KickmessVSTParams {
    fn param(&self, p: Param) -> f32 {
        if let Some(af) = self.atom_float(p) {
            af.get()
        } else {
            0.0
        }
    }
}

fn new_default_atom(ps: &mut ParamSet, p: Param) -> AtomicFloat {
    let idx = ps.idx_of(p).expect("Having a parameter here!");
    AtomicFloat::new(ps.definition(idx).unwrap().default_p())
}

impl Default for KickmessVSTParams {
    fn default() -> KickmessVSTParams {
        let mut ps        = ParamSet::new();
        let mut public_ps = ParamSet::new();
        OpKickmess::init_params(&mut ps, &mut public_ps);

        KickmessVSTParams {
            freq_start:      new_default_atom(&mut ps, Param::Freq1),
            freq_end:        new_default_atom(&mut ps, Param::Freq2),
            length:          new_default_atom(&mut ps, Param::Decay1),
            dist_start:      new_default_atom(&mut ps, Param::Dist1),
            dist_end:        new_default_atom(&mut ps, Param::Dist2),
            dist_gain:       new_default_atom(&mut ps, Param::Gain1),
            env_slope:       new_default_atom(&mut ps, Param::Env1),
            noise:           new_default_atom(&mut ps, Param::Noise1),
            freq_slope:      new_default_atom(&mut ps, Param::Release1),
            env_release:     new_default_atom(&mut ps, Param::Release2),
            freq_note_start: new_default_atom(&mut ps, Param::S1),
            freq_note_end:   new_default_atom(&mut ps, Param::S2),
            phase_offs:      new_default_atom(&mut ps, Param::Phase1),
            ps,
            public_ps,
        }
    }
}

impl PluginParameters for KickmessVSTParams {
    fn get_parameter(&self, index: i32) -> f32 {
        self.public_ps.get_raw(index as usize, self)
    }

    fn set_parameter(&self, index: i32, val: f32) {
        if let Some(pd) = self.public_ps.definition(index as usize) {
            if let Some(af) = self.atom_float(pd.param()) {
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
