use crate::helpers::*;

pub trait OscillatorInputParams{
    fn freq(&self)          -> f32;
    fn waveform(&self)      -> f32;
    fn pulse_width(&self)   -> f32;
    fn detune(&self)        -> f32;
    fn unison(&self)        -> f32;

    fn op1_ratio(&self)     -> f32;
    fn op2_freq(&self)      -> f32;
    fn op1_self(&self)      -> f32;
    fn op2_self(&self)      -> f32;
    fn op1_op2(&self)       -> f32;
    fn op2_op1(&self)       -> f32;
}

pub struct PolyBlepOscillator {
    srate:       f64,
    phase:       f64,
    init_phase:  f64,
    last_output: f64,
}

enum Waveform {
    Sin,
    Tri,
    Saw,
    Sqr,
}

fn sqr(x: f64) -> f64 { x * x }

// PolyBLEP by Tale
// (slightly modified)
// http://www.kvraudio.com/forum/viewtopic.php?t=375517
// from http://www.martin-finke.de/blog/articles/audio-plugins-018-polyblep-oscillator/
//
// default for `pw' should be 1.0, it's the pulse width
// for the square wave.
fn poly_blep(t: f64, dt: f64) -> f64 {
    if t < dt {
        let t = t / dt;
        2. * t - sqr(t) - 1.

    } else if t > (1.0 - dt) {
        let t = (t - 1.0) / dt;
        sqr(t) + 2. * t + 1.

    } else {
        0.
    }
}

impl PolyBlepOscillator {
    pub fn new(init_phase: f64) -> Self {
        Self {
            srate:       0.0,
            phase:       0.0,
            last_output: 0.0,
            init_phase,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate as f64;
    }

    pub fn reset(&mut self) {
        self.phase       = self.init_phase;
        self.last_output = 0.0;
    }

    pub fn next_sin(&mut self) -> f64 {
        fast_sin(self.phase * 2.0 * std::f64::consts::PI)
    }

    pub fn next_tri(&mut self) -> f64 {
        let value = -1.0 + (2.0 * self.phase);
        2.0 * (value.abs() - 0.5)
    }

    pub fn next_saw(&mut self) -> f64 {
        (2.0 * self.phase) - 1.0
    }

    pub fn next_sqr(&mut self, pw: f64) -> f64 {
        if self.phase < pw { 1.0 }
        else { -1.0 }
    }

    pub fn next<P: OscillatorInputParams>(&mut self, params: &P, detune: f64) -> f32 {
        let freq = params.freq() as f64;
        let phase_inc = (freq + detune * freq) / self.srate;

        let wave = params.waveform();

        let waveform =
                 if wave < 0.25 { Waveform::Sin }
            else if wave < 0.5  { Waveform::Tri }
            else if wave < 0.75 { Waveform::Saw }
            else                { Waveform::Sqr };

        let sample =
            match waveform {
                Waveform::Sin => self.next_sin(),
                Waveform::Tri => {
                    let mut sample = self.next_sqr(0.5);
                    sample += poly_blep(self.phase, phase_inc);
                    sample -= poly_blep((self.phase + 0.5).fract(), phase_inc);

                    // leaky integrator: y[n] = A * x[n] + (1 - A) * y[n-1]
                    sample =
                        phase_inc * sample
                        + (1.0 - phase_inc) * self.last_output;
                    self.last_output = sample;

                    // the signal is a bit too weak, we need to amplify it
                    // or else the volume diff between the different waveforms
                    // is too big:
                    sample * 4.0
                },
                Waveform::Saw => {
                    let mut sample = self.next_saw();
                    sample -= poly_blep(self.phase, phase_inc);
                    sample
                },
                Waveform::Sqr => {
                    let pw = params.pulse_width() as f64;
                    let pw = (0.1 * pw) + ((1.0 - pw) * 0.5);
                    let mut sample = self.next_sqr(pw);
                    sample += poly_blep(self.phase, phase_inc);
                    sample -= poly_blep((self.phase + (1.0 - pw)).fract(), phase_inc);
                    sample
                },
            };

        self.phase += phase_inc;
        self.phase = self.phase.fract();

        sample as f32
    }
}

pub struct UnisonBlep {
    oscs: Vec<PolyBlepOscillator>,
    dc_block: crate::filter::DCBlockFilter,
}

impl UnisonBlep {
    pub fn new(max_unison: usize) -> Self {
        let mut oscs = vec![];
        let mut rng = RandGen::new();

        let dis_init_phase = 0.05;
        for i in 0..(max_unison + 1) {
            // randomize phases so we fatten the unison, get
            // less DC and not an amplified signal until the
            // detune desyncs the waves.
            // But no random phase for first, so we reduce the click
            let init_phase =
                if i == 0 { 0.0 } else { rng.next_open01() };
            oscs.push(PolyBlepOscillator::new(init_phase));
        }

        Self {
            oscs,
            dc_block: crate::filter::DCBlockFilter::new(),
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.dc_block.set_sample_rate(srate);
        for o in self.oscs.iter_mut() {
            o.set_sample_rate(srate);
        }
    }

    pub fn reset(&mut self) {
        self.dc_block.reset();
        for o in self.oscs.iter_mut() {
            o.reset();
        }
    }

    pub fn next<P: OscillatorInputParams>(&mut self, params: &P) -> f32 {
        let unison =
            (params.unison().floor() as usize)
            .min(self.oscs.len() - 1);
        let detune = params.detune() as f64;

        let mix = 1.0 / ((unison + 1) as f32);

        let mut s = mix * self.oscs[0].next(params, 0.0);

        for u in 0..unison {
            let detune_factor =
                detune * (((u / 2) + 1) as f64
                          * if (u % 2) == 0 { 1.0 } else { -1.0 });
            s += mix * self.oscs[u + 1].next(params, detune_factor * 0.01);
        }

        self.dc_block.next(s)
    }
}

pub struct FMOscillator {
    srate:       f64,
    phase1:      f64,
    phase2:      f64,
    op1_fb:      f64,
    op2_fb:      f64,
}

impl FMOscillator {
    pub fn new() -> Self {
        Self {
            srate: 44100.0,
            phase1: 0.0,
            phase2: 0.0,
            op1_fb: 0.0,
            op2_fb: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, srate: f32) {
        self.srate = srate as f64;
    }

    pub fn reset(&mut self) {
        self.op1_fb = 0.0;
        self.op2_fb = 0.0;
        self.phase1 = 0.0;
        self.phase2 = 0.0;
    }

    pub fn next<P: OscillatorInputParams>(&mut self, params: &P) -> f32 {
        let freq2      = params.op2_freq() as f64;
        let freq1      = freq2 * (params.op1_ratio() as f64);
        let phase1_inc =
            (freq1
             + params.op1_self() as f64 * self.op1_fb
             + params.op2_op1()  as f64 * self.op2_fb) / self.srate;
        let phase2_inc =
            (freq2
             + params.op2_self() as f64 * self.op2_fb
             + params.op1_op2()  as f64 * self.op1_fb) / self.srate;

        let s_op1 = fast_sin(self.phase1 * 2.0 * std::f64::consts::PI);
        let s_op2 = fast_sin(self.phase2 * 2.0 * std::f64::consts::PI);

        self.op1_fb = s_op1;
        self.op2_fb = s_op2;

        self.phase1 += phase1_inc;
        self.phase1 = self.phase1.fract();
        self.phase2 += phase2_inc;
        self.phase2 = self.phase2.fract();

        s_op2 as f32
    }
}
