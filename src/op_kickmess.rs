use crate::proc::*;
use crate::helpers::*;

pub struct Op_GainDistortion {
    gain_coef:      f32,
    distort_gain:   f32,
    distort_thresh: f32,
}

impl Op_GainDistortion {
    pub fn new() -> Self {
        Self {
            gain_coef:      0.0,
            distort_gain:   1.0,
            distort_thresh: 0.0
        }
    }
}

impl MonoProcessor for Op_GainDistortion {
    fn init_params(ps: &mut ParamSet) {
        ps.add(ParamDefinition::from(Param::Gain1,      -90.0, 24.0, 0.0));
        ps.add(ParamDefinition::from(Param::Gain2,      0.1,    5.0, 1.0));
        ps.add(ParamDefinition::from(Param::Threshold1, 0.0,  100.0, 0.0));
    }

    fn read_params(&mut self, ps: &ParamSet, pp: &dyn ParamProvider) {
        let gain            = ps.get(0, pp);
        self.gain_coef      = gain2coef(gain);
        self.distort_gain   = ps.get(1, pp);
        self.distort_thresh = ps.get(2, pp);
    }

    fn process(&mut self, l: &mut dyn Channel) {
        println!("PROC: {} / {} / {}", self.gain_coef, self.distort_gain, self.distort_thresh);

        l.process(&|i: &[f32], o: &mut [f32]| {
            for (is, os) in i.iter().zip(o) {
                *os = *is * self.gain_coef;
            }
        });
    }
}

// gain: 24.0 - -90.0   default = 0.0
fn gain2coef(gain: f32) -> f32 {
    if gain > -90.0 {
        10.0_f32.powf(gain * 0.05)
    } else {
        0.0
    }
}

