use crate::proc::*;

pub const help_texts : [(&str, &str); 19] = [
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
    ("Filter 1 Resonance", ""),
    ("Filter 1 Type",
        "low pass:  0.0  - 0.33\n\
         high pass: 0.33 - 0.66\n\
         band pass: 0.66 - 1.0 \n\
        "),
    ("Filter 1 Overdrive", ""),
    ("Filter 1 On/Off", ""),
];

macro_rules! param_model {
    ($x: ident) => {
        //  scope  name         exp/lin smooth   idx  min    max     def    width  prec  name
        $x!{public freq_start      exp no_smooth 0,   5.0,   3000.0, 150.0,     7,    2, "Start Freq."}
        $x!{public freq_end        exp no_smooth 1,   5.0,   2000.0,  40.0,     7,    2, "End Freq."}
        $x!{public f_env_release   exp no_smooth 2,   5.0,   5000.0, 440.0,     6,    1, "Length"}
        $x!{public dist_start      lin smooth    3,   0.0,   100.0,    0.8,     4,    2, "Dist. Start"}
        $x!{public dist_end        lin smooth    4,   0.0,   100.0,    0.8,     4,    2, "Dist. End"}
        $x!{public gain            lin smooth    5,   0.0,   2.0,      1.0,     4,    2, "Gain"}
        $x!{public env_slope       lin smooth    6,   0.01,  1.0,    0.163,     5,    3, "Env. slope"}
        $x!{public freq_slope      lin smooth    7,   0.001, 1.0,     0.06,     5,    3, "Freq. slope"}
        $x!{public noise           exp smooth    8,   0.0,   1.0,      0.0,     4,    2, "Tone/Noise"}
        $x!{public freq_note_start lin no_smooth 9,   0.0,   1.0,      0.0,     3,    1, "Note > Start Freq"}
        $x!{public freq_note_end   lin no_smooth 10,  0.0,   1.0,      0.0,     3,    1, "Note > End Freq"}
        $x!{public env_release     lin no_smooth 11,  1.0,1000.0,      5.0,     4,    2, "Env Release"}
        $x!{public phase_offs      lin smooth    12,  0.0,   1.0,      0.0,     4,    2, "Click"}
        $x!{public dist_on         lin no_smooth 13,  0.0,   1.0,      0.0,     3,    1, "Dist. On"}
        $x!{public f1_cutoff       exp smooth    14, 20.0,   22050.0,  5000.0,  7,    1, "F1 Cutoff"}
        $x!{public f1_res          lin smooth    15,  0.0,   1.0,      0.0,     4,    2, "F1 Res"}
        $x!{public f1_type         lin no_smooth 16,  0.0,   1.0,      0.0,     3,    1, "F1 Type"}
        $x!{public f1_drive        lin smooth    17,  0.0,   5.0,      1.0,     4,    2, "F1 Type"}
        $x!{public f1_on           lin no_smooth 18,  0.0,   1.0,      0.0,     3,    1, "F1 On"}
        $x!{public o1_gain         exp smooth    19,  0.0,   2.0,      0.0,     4,    2, "O1 Gain"}
        $x!{public o1_waveform     lin smooth    20,  0.0,   1.0,      0.0,     4,    2, "O1 Wave"}
        $x!{public o1_pw           lin smooth    21,  0.0,   1.0,      1.0,     4,    2, "O1 PW"}
        $x!{public o1_unison       lin no_smooth 22,  0.0,  10.5,      0.0,     1,    0, "O1 Unison"}
        $x!{public o1_detune       lin smooth    23,  0.0,   1.0,      0.01,    5,    3, "O1 Detune"}

        $x!{public o1fm_freq       exp smooth    24,  0.0,5000.0,     300.0,    5,    3, "OP1 Freq"}
        $x!{public o1fm_self       lin smooth    25,  0.0,  10.0,       0.0,    5,    3, "OP1 Self"}
        $x!{public o1fm_o2_mod     lin smooth    26,  0.0,  10.0,       0.5,    5,    3, "OP1>OP2"}
        $x!{public o2fm_o1_mod     lin smooth    27,  0.0,  10.0,       0.0,    5,    3, "OP2>OP1"}
        $x!{public o2fm_freq       exp smooth    28,  0.0,5000.0,     500.0,    5,    3, "OP2 Freq"}
        $x!{public o2fm_self       lin smooth    29,  0.0,  10.0,       0.0,    5,    3, "OP2 Self"}
        $x!{public o2fm_gain       lin smooth    30,  0.0,  10.0,       0.0,    5,    3, "OP2 Gain"}

        $x!{private phase_test     lin smooth    31,  0.0,   1.0,      0.0,     5,    2, "Click2"}
    }
}

pub struct ParamModel<'a> {
    v: &'a [f32],
}

macro_rules! param_impl_accessors {
    ($_:ident $name:ident $e:ident $s:ident $idx:expr, $($tt:tt)*) => {
        impl ParamModel<'_> {
            pub fn $name(&self) -> f32 { self.v[$idx] }
        }
    }
}

impl<'a> ParamModel<'a> {
    pub fn new(v: &'a [f32]) -> Self {
        Self { v }
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

param_model!{param_impl_accessors}
