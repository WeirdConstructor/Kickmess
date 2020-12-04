#![allow(warnings)]
pub mod proc;
pub mod helpers;
mod op_kickmess;
mod env;
mod editor;
pub mod pugl;
pub mod ui;
pub mod baseview;

use proc::Channel;
use proc::{ParamProvider, Param, ParamSet, MonoProcessor, MonoVoice};
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
        }
    }
}

struct Kickmess {
    host:      HostCallback,
    params:    Arc<KickmessVSTParams>,
    voices:    Vec<OpKickmess>,
}

impl Plugin for Kickmess {
    fn new(host: HostCallback) -> Self {
        let mut voices = vec![];
        for _ in 0..10 {
            voices.push(OpKickmess::new());
        }

        Self {
            host,
            voices,
            params: Arc::new(KickmessVSTParams::default()),
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
            parameters:   self.params.ps.param_count() as i32,
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
                        let mut playing = 0;
                        let mut release = 0;
                        for voice in self.voices.iter_mut() {
                            if voice.is_playing() {
                                playing += 1;
                                if voice.in_release() {
                                    release += 1;
                                }
                            }
                        }
                        //d// println!("P VOICES: p={}, r={}", playing, release);

                        for (i, voice) in self.voices.iter_mut().enumerate() {
                            if !voice.is_playing() {
                                //d// println!("[VOICE {}] START", i);
                                voice.start_note(
                                    delta_frames as usize,
                                    note_to_freq(data[1] as f32),
                                    data[2] as f32);
                                break;
                            }
                        }

                    } else if data[0] == 128 {
                        for (i, voice) in self.voices.iter_mut().enumerate() {
                            if voice.is_playing() && !voice.in_release() {
                                //d// println!("[VOICE {}] END", i);
                                voice.end_note(delta_frames as usize);
                                break;
                            }
                        }
                    }

                    //d// println!("MIDI: {:?}", data);
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
        let mut ps = ParamSet::new();
        OpKickmess::init_params(&mut ps);

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
            ps,
        }
    }
}

impl PluginParameters for KickmessVSTParams {
    fn get_parameter(&self, index: i32) -> f32 {
        self.ps.get_raw(index as usize, self)
    }

    fn set_parameter(&self, index: i32, val: f32) {
        if let Some(pd) = self.ps.definition(index as usize) {
            if let Some(af) = self.atom_float(pd.param()) {
                af.set(val);
            }
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        if index > self.ps.param_count() as i32 {
            return "".to_string();
        }

        let v = self.get_parameter(index);
        let pd = self.ps.definition(index as usize).unwrap();
        format!("{} >= {:.2} >= {}", pd.min(), pd.map(v), pd.max())
    }

    fn get_parameter_name(&self, index: i32) -> String {
        if let Some(pd) = self.ps.definition(index as usize) {
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
