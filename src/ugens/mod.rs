use std::f32::consts::PI;
use std::ops::Rem;

/// Returns the signal value (between -1 and 1)
/// # Arguments
///
///  * `time_within_period` - between 0 and 1
pub type OscFunction = fn(time_within_period: f32) -> f32;

pub struct FunctionOsc {
    sample_rate: u32,
    osc_function: OscFunction,
    freq: f32,
    pos: usize,
}

impl FunctionOsc {
    pub fn new(sample_rate: u32, osc_function: OscFunction) -> Self {
        FunctionOsc {
            sample_rate,
            osc_function,
            freq: 220.0,
            pos: 0,
        }
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.freq = freq;
    }

    pub fn compute(&mut self) -> f32 {
        let time = (self.pos as f32) / (self.sample_rate as f32);
        let period_length = 1.0 / self.freq;
        let x = (time % period_length) / period_length;
        let result = (self.osc_function)(x);
        self.pos += 1;
        result
    }
}

pub struct UGenFactory {
    sample_rate: u32,
}

impl UGenFactory {
    pub fn new(sample_rate: u32) -> Self {
        UGenFactory { sample_rate }
    }

    pub fn sin(&self) -> FunctionOsc {
        FunctionOsc::new(self.sample_rate, |x| (x * 2.0 * PI).sin())
    }

    pub fn saw(&self) -> FunctionOsc {
        FunctionOsc::new(self.sample_rate, |x| (x - 0.5) * 2.0)
    }

    pub fn triangle(&self) -> FunctionOsc {
        FunctionOsc::new(self.sample_rate, |x| match x {
            0.0..=0.25 => x * 4.0,
            0.25..=0.75 => 1.0 - (x - 0.25) * 4.0,
            0.75..=1.0 => (x - 0.75) * 4.0 - 1.0,
            _ => panic!("Unsupported input {}", x),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ugens::FunctionOsc;

    #[test]
    fn it_works() {
        let mut osc = FunctionOsc::new(4, |x| x);
        osc.freq = 1.0;
        assert_eq!(osc.compute(), 0.0);
        assert_eq!(osc.compute(), 0.25);
        assert_eq!(osc.compute(), 0.5);
        assert_eq!(osc.compute(), 0.75);
        assert_eq!(osc.compute(), 0.0);
        assert_eq!(osc.compute(), 0.25);
        assert_eq!(osc.compute(), 0.5);
        assert_eq!(osc.compute(), 0.75);
    }

    #[test]
    fn freq_is_2() {
        let mut osc = FunctionOsc::new(4, |x| x);
        osc.freq = 2.0;
        assert_eq!(osc.compute(), 0.0);
        assert_eq!(osc.compute(), 0.5);
        assert_eq!(osc.compute(), 0.0);
        assert_eq!(osc.compute(), 0.5);
    }

    #[test]
    fn test_osc_function() {
        let mut osc = FunctionOsc::new(4, |x| x * 10.0);
        osc.freq = 2.0;
        assert_eq!(osc.compute(), 0.0);
        assert_eq!(osc.compute(), 5.0);
        assert_eq!(osc.compute(), 0.0);
        assert_eq!(osc.compute(), 5.0);
    }
}
