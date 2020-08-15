use crate::engines::test::TestEngine;
use crate::engines::young::Young;
use crate::synth_engine::SynthEngine;

pub mod test;
pub mod young;

pub fn create_enginge(program: u8, sample_rate: u32) -> Box<dyn SynthEngine> {
    match program {
        1 => TestEngine::new(sample_rate).boxit(),
        2 => Young::new(sample_rate).boxit(),
        _ => {unimplemented!()}
    }
}

trait Boxable {
    fn boxit(self) -> Box<Self>;
}

impl<T> Boxable for T {
    fn boxit(self) -> Box<Self> {
        Box::new(self)
    }
}