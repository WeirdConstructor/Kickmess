use crate::proc::*;

pub const help_texts : [(&str, &str); 48] = [
    ("Start Frequency",
        "This is the starting frequency of the frequency envelope."),
    ("End Frequency",
        "This is the ending frequency of the frequency envelope."),
    ("Length",
        "The lengths of the frequency and amplitude envelope in milliseconds."),
    ("Distortion start amount",
        "Distortion has it's own linear envelope.\n\
         You can have different start and ending amount of\n\
         the distortion envelope."),
    ("Distortion end amount",
        "Distortion has it's own linear envelope.\n\
         You can have different start and ending amount of\n\
         the distortion envelope."),
    ("Gain",
        "Additional gain applied to the output of the synthesizer."),
    ("Envelope Slope",
        "The slope of the amplitude envelope.\n\
         You can go from linear to exponential."),
    ("Frequency Envelope Slope",
        "The slope of the frequency envelope.\n\
         You can go from linear to exponential."),
    ("Noise/Tone Balance",
        "The balance between tone (0.0) and noise (1.0)."),
    ("Note pitch is Start frequency",
        "If you enable this, the frequency will start with the\n\
         pitch of the played MIDI note."),
    ("Note pitch is End frequency",
        "If you enable this, the frequency will end with the\n\
         pitch of the played MIDI note."),
    ("Env Release",
        "There is a second release envelope that affects the amplifier.\n\
         It is started when the MIDI note off event is received.\n\
         This parameter defines the length of that release."),
    ("Click Amount",
        "This value will cut the phase of the sine wave,\n\
         causing an audible extra 'click' at the start of the note."),
    ("Distortion",
        "If the distortion is enabled, the 'Start' and 'End' amounts will\n\
         define the amount of distortion at the beginning and end of the\n\
         envelope."),
    ("Filter 1 Cutoff Frequency", ""),
    ("Filter 1 Resonance",        ""),
    ("Filter 1 Overdrive",        ""),
    ("Main Gain",                 ""),
    ("Env 1 Attack",              ""),
    ("Env 1 Decay",               ""),
    ("Env 1 Sustain",             ""),
    ("Env 1 Release",             ""),

    // Megamess Parameters
    ("Oscillator 1 Gain",            ""),
    ("Oscillator 1 Wave",            ""),
    ("Oscillator 1 Pulsewidth",      ""),
    ("Oscillator 1 Unison",          ""),
    ("Oscillator 1 Detune",          ""),
    ("FM OP1 Frequency Ratio",       ""),
    ("FM OP1 Self Modulation",       ""),
    ("FM OP1 -> OP2 Modulation",     ""),
    ("FM OP2 -> OP1 Modulation",     ""),
    ("FM OP2 Fixed Frequency",       ""),
    ("FM OP2 Self Modulation",       ""),
    ("FM OP2 Gain",                  ""),
    ("LFO 1 - Frequency",            ""),
    ("LFO 1 - Frequency Multiplier", ""),
    ("LFO 1 - Waveform",             ""),
    ("LFO 1 - Pulsewidth",           ""),
    ("LFO 1 - Phase Offset",         ""),
    ("Modulator 1 - Amount",         ""),
    ("Modulator 1 - Slope",          ""),

    // Private Parameters
    ("Filter 1 Type",
        "low pass:  0.0  - 0.33\n\
         high pass: 0.33 - 0.66\n\
         band pass: 0.66 - 1.0 \n\
        "),
    ("Filter 1 On/Off",                      ""),
    ("MIDI Channel",                         ""),
    ("FM OP2 Frequency Mode",                ""),
    ("Modulator 1 - Modulation Source",      ""),
    ("Modulator 1 - Modulation Destination", ""),
    ("Modulator 1 - Modulation Function",    ""),
];

#[cfg(not(feature="mega"))]
macro_rules! define_constants {
    () => {
        pub const PUB_PARAM_COUNT : usize = 22;
        pub const PARAM_COUNT     : usize = 25;
        macro_rules! ppc {
            ($x: expr) => {
                ($x + crate::param_model::PUB_PARAM_COUNT)
            }
        }
    }
}

#[cfg(feature="mega")]
macro_rules! define_constants {
    () => {
        pub const PUB_PARAM_COUNT : usize = 41;
        pub const PARAM_COUNT     : usize = 48;
        macro_rules! ppc {
            ($x: expr) => {
                ($x + crate::param_model::PUB_PARAM_COUNT)
            }
        }
    }
}

define_constants!{}


macro_rules! mega_params {
    ($x: ident) => {
        //  scope   name         exp/lin smooth        idx  min    max     def    width  prec  label
        $x!{public  o1_gain         exp smooth         22,  0.0,   2.0,      0.0,     4,    2, "O1 Gain"}
        $x!{public  o1_waveform     lin no_smooth      23,  0.0,   1.0,      0.0,     4,    2, "O1 Wave"}
        $x!{public  o1_pw           lin smooth         24,  0.0,   1.0,      1.0,     4,    2, "O1 PW"}
        $x!{public  o1_unison       lin no_smooth      25,  0.0,  10.5,      0.0,     1,    0, "O1 Unison"}
        $x!{public  o1_detune       lin smooth         26,  0.0,   1.0,      0.01,    5,    3, "O1 Detune"}

        $x!{public  o1fm_ratio      exp smooth         27,  0.0,  30.0,       2.0,    5,    3, "OP1 Ratio"}
        $x!{public  o1fm_self       exp smooth         28,  0.0,30000.0,      0.0,    4,    2, "OP1 Self Hz"}
        $x!{public  o1fm_o2_mod     exp smooth         29,  0.0,30000.0,    100.0,    4,    2, "OP1>OP2 Hz"}
        $x!{public  o2fm_o1_mod     exp smooth         30,  0.0,30000.0,      0.0,    4,    2, "OP2>OP1 Hz"}
        $x!{public  o2fm_freq       exp smooth         31,  0.0,30000.0,    500.0,    4,    2, "OP2 Freq Hz"}
        $x!{public  o2fm_self       exp smooth         32,  0.0,30000.0,      0.0,    4,    2, "OP2 Self Hz"}
        $x!{public  o2fm_gain       lin smooth         33,  0.0,   2.0,       0.0,    5,    3, "OP2 Gain"}
        $x!{private o2fm_mode       lin no_smooth ppc!(3),  0.0,   1.0,       0.0,    3,    1, "OP2 Mode"}

        $x!{public  lfo1_freq      exp4 smooth         34,  0.0, 100.0,       1.0,    5,    3, "LFO1 Freq"}
        $x!{public  lfo1_fmul       lin smooth         35,  0.1, 100.0,       1.0,    5,    3, "LFO1 FMul"}
        $x!{public  lfo1_wave       lin no_smooth      36,  0.0,   1.0,       0.0,    3,    1, "LFO1 Wave"}
        $x!{public  lfo1_pw         lin smooth         37,  0.01, 0.99,       0.5,    3,    1, "LFO1 PW"}
        $x!{public  lfo1_phase      lin smooth         38,  0.0,   1.0,       0.0,    3,    1, "LFO1 Phase"}

        $x!{public   m1_amount      lin smooth         39,  0.0,   1.0,       1.0,    4,    2, "Mod1 Amt"}
        $x!{public   m1_slope       lin smooth         40,  0.0,   1.0,       0.0,    5,    3, "Mod1 Slope"}

        $x!{private  m1_src_id      lin no_smooth ppc!(4),  0.0,1000.0,      -1.0,    1,    0, "Mod1 Src"}
        $x!{private  m1_dest_id     lin no_smooth ppc!(5),  0.0,   1.0,       0.0,    1,    0, "Mod1 Dest"}
        $x!{private  m1_fun         lin no_smooth ppc!(6),  0.0,   1.0,       0.0,    3,    1, "Mod1 Fun"}
    }
}

macro_rules! param_model {
    ($x: ident) => {
//        use crate::param_model::PUB_PARAM_COUNT as PPC;

        //  scope   name         exp/lin smooth        idx  min    max     def    width  prec  "mod./vst label"
        $x!{public  freq_start      exp no_smooth      0,   5.0,   3000.0, 150.0,     4,    2, "Start Freq."}
        $x!{public  freq_end        exp no_smooth      1,   5.0,   2000.0,  40.0,     4,    2, "End Freq."}
        $x!{public  f_env_release   exp no_smooth      2,   5.0,   5000.0, 440.0,     3,    1, "Length"}
        $x!{public  dist_start      lin smooth         3,   0.0,   100.0,    0.8,     4,    2, "Dist. Start"}
        $x!{public  dist_end        lin smooth         4,   0.0,   100.0,    0.8,     4,    2, "Dist. End"}
        $x!{public  gain            lin smooth         5,   0.0,   2.0,      1.0,     4,    2, "Gain"}
        $x!{public  env_slope       lin smooth         6,   0.01,  1.0,    0.163,     5,    3, "Env. slope"}
        $x!{public  freq_slope      lin smooth         7,   0.001, 1.0,     0.06,     5,    3, "Freq. slope"}
        $x!{public  noise           exp smooth         8,   0.0,   1.0,      0.0,     4,    2, "Tone/Noise"}
        $x!{public  freq_note_start lin no_smooth      9,   0.0,   1.0,      0.0,     3,    1, "Note > Start Freq"}
        $x!{public  freq_note_end   lin no_smooth      10,  0.0,   1.0,      0.0,     3,    1, "Note > End Freq"}
        $x!{public  env_release     lin no_smooth      11,  1.0,1000.0,      5.0,     4,    2, "Env Release"}
        $x!{public  phase_offs      lin smooth         12,  0.0,   1.0,      0.0,     4,    2, "Click"}
        $x!{public  dist_on         lin no_smooth      13,  0.0,   1.0,      0.0,     3,    1, "Dist. On"}

        $x!{public  f1_cutoff       exp smooth         14, 20.0,   22050.0,  5000.0,  3,    1, "F1 Cutoff"}
        $x!{public  f1_res          lin smooth         15,  0.0,   1.0,      0.0,     4,    2, "F1 Res"}
        $x!{public  f1_drive        lin smooth         16,  0.0,   5.0,      1.0,     4,    2, "F1 Drive"}
        $x!{public  main_gain       exp smooth         17,  0.0,   2.0,      1.0,     5,    3, "Main Gain"}

        $x!{public  e1_attack       exp smooth         18,  0.0,5000.0,     50.0,     5,    3, "E1 Attack"}
        $x!{public  e1_decay        exp smooth         19,  0.0,5000.0,    100.0,     5,    3, "E1 Decay"}
        $x!{public  e1_sustain      lin smooth         20,  0.0,   1.0,     0.75,     4,    2, "E1 Sustain"}
        $x!{public  e1_release      exp smooth         21,  0.0,5000.0,     50.0,     5,    3, "E1 Decay"}

        $x!{private f1_type         lin no_smooth ppc!(0),  0.0,   1.0,      0.0,     3,    1, "F1 Type"}
        $x!{private f1_on           lin no_smooth ppc!(1),  0.0,   1.0,      0.0,     3,    1, "F1 On"}
        $x!{private midi_chan       lin no_smooth ppc!(2),  0.0,  15.9,       0.0,    2,    0, "Midi Chan"}

        #[cfg(feature="mega")]
        mega_params!{$x}
    }
}

pub struct ParamModel<'a> {
    v: &'a [f32],
}

pub struct ParamModelMut {
    idx: usize,
    v: [[f32; PARAM_COUNT]; 2],
}

macro_rules! param_impl_accessors {
    ($_:ident $name:ident $e:ident $s:ident $idx:expr, $($tt:tt)*) => {
        impl ParamModel<'_> {
            pub fn $name(&self) -> f32 { self.v[$idx] }
        }

        impl ParamModelMut {
            pub fn $name(&self) -> f32 { self.v[self.idx][$idx] }
        }
    }
}

pub mod pid {
    macro_rules! param_ids {
        ($_:ident $name:ident $e:ident $s:ident $idx:expr, $($tt:tt)*) => {
            pub const $name : usize = $idx;
        }
    }
    param_model!{param_ids}
}

pub fn create_mod_params() -> Vec<(usize, &'static str)> {
    let mut ret = vec![];

    macro_rules! param_deserialize {
        ($_:ident $name:ident $e:ident smooth $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
            ret.push(($idx, $lbl));
        };
        ($_:ident $name:ident $e:ident no_smooth $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
            {}
        };
    }

    param_model!{param_deserialize}

    ret
}

pub fn deserialize_preset<F: Fn(usize, f32)>(preset: &[u8], out: F) {
    let mut preset_data : Vec<(String, f32)> = vec![];

    let mut data = String::from_utf8_lossy(preset);
    let fields : Vec<&str> = data.split(";").collect();

    let mut start_params = false;
    for f in fields.iter() {
        let part = f.trim();

        if start_params {
            let par : Vec<&str> = part.split("=").collect();
            preset_data.push((
                (*par.get(0).unwrap_or_else(|| &"?")).to_string(),
                par.get(1).unwrap_or_else(|| &"0").parse::<f32>().unwrap_or(0.0)
            ));

        } else {
            if f == &"!PARAMS" {
                start_params = true;
            }
        }
    }

    macro_rules! param_deserialize {
        (public $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
            for (name, value) in preset_data.iter() {
                if name == stringify!(pub:$name) {
                    (out)($idx, *value);
                }
            }
        };
        (private $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
            for (name, value) in preset_data.iter() {
                if name == stringify!(priv:$name) {
                    (out)($idx, *value);
                }
            }
        };
    }

    param_model!{param_deserialize}
}

pub fn serialize_preset(pp: &dyn ParamProvider) -> Vec<u8> {
    let mut out = String::new();

    out += "!PARAMS;\n";

    macro_rules! param_serialize {
        (public $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
            out += stringify!(pub:$name);
            out += "=";
            out += &pp.param($idx).to_string();
            out += ";\n";
        };
        (private $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
            out += stringify!(priv:$name);
            out += "=";
            out += &pp.param($idx).to_string();
            out += ";\n";
        };
    }

    param_model!{param_serialize}

    out.into_bytes()
}

impl<'a> ParamModel<'a> {
    pub fn new(v: &'a [f32]) -> Self {
        Self { v }
    }

    pub fn is_public(id: usize) -> bool {
        macro_rules! param_add_ps {
            (private $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
                if id == $idx { return false; }
            };
            (public $($tt:tt)*) => {
            }
        }
        true
    }

    pub fn init_public_set(ps: &mut ParamSet) {
        macro_rules! param_add_ps {
            (public $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
                ps.add(ParamDefinition::from($idx, $min, $max, $def, $width, $prec, $lbl).$e().$s());
            };
            (private $($tt:tt)*) => {
            }
        }

        param_model!{param_add_ps}
    }

    pub fn init_private_set(ps: &mut ParamSet) {
        macro_rules! param_add_ps_priv {
            ($_:ident $name:ident $e:ident $s:ident $idx:expr, $min:expr, $max:expr, $def:expr, $width:expr, $prec:expr, $lbl:expr) => {
                ps.add(ParamDefinition::from($idx, $min, $max, $def, $width, $prec, $lbl).$e().$s());
            }
        }

        param_model!{param_add_ps_priv}
    }
}

#[inline]
pub fn mod_function(mod_val: f32, fun_select: f32, mod_amount: f32, mod_slope: f32) -> f32 {
    let mod_val =
        if mod_slope < 0.5 {
            mod_val.powf((0.5 - mod_slope) * 2.0)
        } else {
            (1.0 - mod_val).powf((mod_slope - 0.5) * 2.0)
        };

//    crate::log::log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//        use std::io::Write;
//        write!(bw, "MF1 mv={}, ms={}, ma={}",
//               mod_val, mod_slope, mod_amount);
//    });

    if fun_select < 0.25 {         // a * x            [0, a]
        mod_amount * mod_val
    } else if fun_select < 0.5 {   // a * (1 - x)      [a, 0]
        mod_amount * (1.0 - mod_val)
    } else if fun_select < 0.75 {  // 1 - a * x        [1, 1 - a]
        1.0 - (mod_amount * mod_val)
    } else {                       // 1 - a * (1 - x)  [1 - a, 1]
        1.0 - mod_amount * (1.0 - mod_val)
    }
}

pub struct ModulatorFun {
    param_id:  f32,
    param_val: f32,
    mod_val:   f32,
}

impl ModulatorFun {
    pub fn new() -> Self {
        Self {
            param_id:  0.0,
            param_val: 0.0,
            mod_val:   0.0,
        }
    }

    #[inline]
    pub fn set_param(&mut self, p: f32) {
        self.param_id = p;
    }

    #[inline]
    pub fn feedback_run(&mut self, pm: &mut ParamModelMut) {
        if self.param_id >= 0.0 {
            self.param_val = pm.getf(self.param_id);
            pm.setf(self.param_id, self.param_val * self.mod_val);
        } else {
            self.param_val = 0.0;
        }
    }

    #[inline]
    pub fn run_mod_fun(&mut self, pm: &mut ParamModelMut, mod_val: f32,
               mod_fun: f32, mod_amount: f32, mod_slope: f32) {

        self.mod_val = mod_function(mod_val, mod_fun, mod_amount, mod_slope);
        if self.param_id >= 0.0 {
//            crate::log::log(|bw: &mut std::io::BufWriter<&mut [u8]>| {
//                use std::io::Write;
//                write!(bw, "MF1 modval={}, paramval={}, res={}",
//                       self.mod_val, self.param_val,
//                       self.param_val * self.mod_val);
//            });
            pm.setf(self.param_id, self.param_val * self.mod_val);
        }
    }
}

// self.modfun1.set_param_id(pid::m1_dest_id);
// self.modfun1.apply(params); // saves param value in param_val, applies prev mod_val
//
// let mod_val = self.lfo1.next();
// --- map mod_val to pid::m1_src_id ---
//
// self.modfun1.run(mod_val, mod_amount, mod_slope);
//
// // takes param_val from earlier, applies new mod_val to it, stores in params
// self.modfun1.post_apply(params);

impl ParamModelMut {
    pub fn new() -> Self {
        let mut v = [[0.0; PARAM_COUNT]; 2];
        Self {
            v,
            idx: 0,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.idx = 0;
        for v in self.v[self.idx].iter_mut() {
            *v = 0.0;
        }
    }

    #[inline]
    pub fn get_prev_frame(&mut self) -> &[f32] {
        &self.v[(self.idx + 1) % 2][..]
    }

    #[inline]
    pub fn swap(&mut self, i: &[f32]) {
        self.idx = (self.idx + 1) % 2;
        for (i, v) in i.iter().zip(self.v[self.idx].iter_mut()) {
            *v = *i;
        }
    }

    #[inline]
    pub fn getf(&mut self, id: f32) -> f32 {
        self.v[self.idx][(id + 0.1).floor() as usize]
    }

    #[inline]
    pub fn setf(&mut self, id: f32, v: f32) {
        self.v[self.idx][(id + 0.1).floor() as usize] = v;
    }
}

param_model!{param_impl_accessors}
