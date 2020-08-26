use soundpipe::factory::Factory;
use soundpipe::soundpipe::midi2cps;
use soundpipe::ugens::oscillators::bl_saw::BlSaw;
use soundpipe::ugens::oscillators::common::MonoOsc;
use soundpipe::Soundpipe;

pub struct UnisonOscillator {
    voices: Vec<BlSaw>,
    detune: f32,
}

impl UnisonOscillator {
    pub fn new(sp: &Soundpipe, voice_number: usize, detune: f32) -> Self {
        UnisonOscillator {
            voices: (1..voice_number).map(|_| sp.bl_saw()).collect(),
            detune,
        }
    }

    pub fn set_note(&mut self, note: f32) {
        let n = self.voices.len() as f32;
        let middle = n / 2.0;
        for (i, voice) in self.voices.iter().enumerate() {
            voice.set_freq(midi2cps(
                note + (i as f32 - middle).abs() / n * 2.0 * self.detune,
            ));
        }
    }

    pub fn compute(&self) -> f32 {
        self.voices.iter().map(|it| it.compute()).sum::<f32>() / (self.voices.len() as f32)
    }
}
