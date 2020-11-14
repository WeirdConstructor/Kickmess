mod effects;
mod helpers;

use effects::Channel;
use effects::{p2range, ParamProvider, Param, ParamSet, Op_GainDistortion, MonoProcessor};

#[macro_use]
extern crate vst;

use vst::util::AtomicFloat;
use vst::api::Events;
use vst::event::{Event, MidiEvent};
use vst::buffer::AudioBuffer;
use vst::plugin::{HostCallback, Category, Info, Plugin, PluginParameters, CanDo};

use std::sync::Arc;

struct AB<'a>((&'a [f32], &'a mut [f32]));


impl<'a> AB<'a> {
    fn from_buffers(bufs: (&'a [f32], &'a mut [f32])) -> Self {
        Self(bufs)
    }
}

impl<'a> Channel for AB<'a> {
    fn process(&mut self, f: &dyn Fn(&[f32], &mut [f32])) {
        f(self.0.0, self.0.1)
    }
}


impl Default for Kickmess {
    fn default() -> Kickmess {
        Kickmess {
            host:   HostCallback::default(),
            params: Arc::new(GainEffectParameters::default()),
            op:     Op_GainDistortion::new(),
        }
    }
}

struct Kickmess {
    host:      HostCallback,
    params:    Arc<GainEffectParameters>,
    op:        Op_GainDistortion,
}

impl Plugin for Kickmess {
    fn new(host: HostCallback) -> Self {
        Self {
            host,
            params: Arc::new(GainEffectParameters::default()),
            op:     Op_GainDistortion::new(),
        }
    }

    fn init(&mut self) {
        helpers::init_cos_tab();
    }

    fn get_info(&self) -> Info {
        Info {
            name:         "Kickmess (VST)".to_string(),
            vendor:       "Weird Constructor".to_string(),
            inputs:       2,
            outputs:      2,
            midi_inputs:  1,
            midi_outputs: 1,
            parameters:   3,
            unique_id:    934843292,
            version:      0001,
            category:     Category::Effect,
            ..Default::default()
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        self.op.read_params(&self.params.ps, &*self.params);

        for buf_tuple in buffer.zip() {
            self.op.process(&mut AB::from_buffers(buf_tuple));
        }
    }

    fn process_events(&mut self, events: &Events) {
        for e in events.events() {
            match e {
                Event::Midi(MidiEvent { data, .. }) => {
                    println!("MIDI: {:?}", data);
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
}

struct GainEffectParameters {
    ps:             ParamSet,
    gain:           AtomicFloat,
    distort_gain:   AtomicFloat,
    distort_thresh: AtomicFloat,
}

impl ParamProvider for GainEffectParameters {
    fn param(&self, p: Param) -> f32 {
        match p {
            Param::Gain1      => self.gain.get(),
            Param::Gain2      => self.distort_gain.get(),
            Param::Threshold1 => self.distort_thresh.get(),
            _                 => 0.0,
        }
    }
}

impl Default for GainEffectParameters {
    fn default() -> GainEffectParameters {
        let mut ps = ParamSet::new();
        Op_GainDistortion::init_params(&mut ps);

        GainEffectParameters {
            gain:           AtomicFloat::new(ps.definition(0).unwrap().default_p()),
            distort_gain:   AtomicFloat::new(ps.definition(1).unwrap().default_p()),
            distort_thresh: AtomicFloat::new(ps.definition(2).unwrap().default_p()),
            ps,
        }
    }
}

impl PluginParameters for GainEffectParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.gain.get(),
            1 => self.distort_gain.get(),
            2 => self.distort_thresh.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        match index {
            0 => self.gain.set(val),
            1 => self.distort_gain.set(val),
            2 => self.distort_thresh.set(val),
            _ => (),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        if index > 2 {
            return "".to_string();
        }

        let v =
            match index {
                0 => self.gain.get(),
                1 => self.distort_gain.get(),
                2 => self.distort_thresh.get(),
                _ => 0.0,
            };

        let pd = self.ps.definition(index as usize).unwrap();
        format!("{} >= {:.2} >= {}", pd.min(), pd.map(v), pd.max())
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Gain",
            1 => "Distort Gain",
            2 => "Distort Thresh",
            _ => "",
        }
        .to_string()
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
