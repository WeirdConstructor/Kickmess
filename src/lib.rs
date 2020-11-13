#[macro_use]
extern crate vst;

use vst::util::AtomicFloat;
use vst::buffer::AudioBuffer;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use std::sync::Arc;

struct GainEffectParameters {
    gain: AtomicFloat,
}

impl Default for Kickmess {
    fn default() -> Kickmess {
        Kickmess {
            params: Arc::new(GainEffectParameters::default()),
        }
    }
}

impl Default for GainEffectParameters {
    fn default() -> GainEffectParameters {
        GainEffectParameters {
            gain: AtomicFloat::new(0.5),
        }
    }
}

#[derive(Default)]
struct Kickmess {
    params: Arc<GainEffectParameters>,
}

impl Plugin for Kickmess {
    fn get_info(&self) -> Info {
        Info {
            name:         "Kickmess (VST)".to_string(),
            vendor:       "Weird Constructor".to_string(),
            inputs:       2,
            outputs:      2,
            midi_inputs:  1,
            midi_outputs: 1,
            parameters:   1,
            unique-id:    934843292,
            version:      0001,
            category:     Category::Effect,
            ..Default::default()
        }
    }

   fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let gain = self.params.gain.get();

        let coef = if gain > -90.0 {
            10.0_f32.powf(gain * 0.05)
        } else {
            0.0
        };

        for (input_buffer, output_buffer) in buffer.zip() {
            for (input_sample, output_sample) in input_buffer.iter().zip(output_buffer) {
                *output_sample = *input_sample * gain;
            }
        }
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

impl PluginParameters for GainEffectParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.gain.get(),
            _ => 0.0,
        }
    }

    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        match index {
            0 => self.gain.set(val),
            _ => (),
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("24 >= {:.2} >= -90", self.gain.get()),
            _ => "".to_string(),
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Gain",
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
