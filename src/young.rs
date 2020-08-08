use midi_message::MidiMessage;
use soundpipe::factory::Factory;
use soundpipe::soundpipe::midi2cps;
use soundpipe::ugens::effects::revsc::Revsc;
use soundpipe::ugens::envelopes::adsr::Adsr;
use soundpipe::ugens::oscillators::bl_saw::BlSaw;
use soundpipe::ugens::oscillators::common::MonoOsc;
use soundpipe::Soundpipe;

use crate::synth_engine::SynthEngine;

pub struct Young {
    adsr: Adsr,
    osc1: BlSaw,
    osc2: BlSaw,
    reverb: Revsc,
    gate: f32,
}

impl Young {
    pub fn new(sample_rate: u32) -> Self {
        let sp = Soundpipe::new(sample_rate as i32);

        let adsr = sp.adsr();
        adsr.set_attack_time(0.01);

        let osc1 = sp.bl_saw();
        let osc2 = sp.bl_saw();

        let reverb = sp.revsc();
        reverb.set_feedback(0.6);
        Young {
            adsr,
            osc1,
            osc2,
            reverb,
            gate: 0.0,
        }
    }
}

impl SynthEngine for Young {
    fn on_midi_message(&mut self, midi_message: MidiMessage) {
        match midi_message {
            MidiMessage::NoteOn(_, midi_note, _) => {
                self.osc1.set_freq(midi2cps(midi_note as f32));
                self.osc2.set_freq(midi2cps((midi_note + 7) as f32));
                self.gate = 1.0;
            }
            MidiMessage::NoteOff(_, _, _) => {
                self.gate = 0.0;
            }
            _ => {}
        }
    }

    fn compute_output(&mut self) -> (f32, f32) {
        let mix = (self.osc1.compute() + self.osc2.compute()) / 2.0;
        let mono = mix * self.adsr.compute(self.gate) * 0.7;
        let reverbed = self.reverb.compute(mono, mono);
        let left = (mono + reverbed.0) / 2.0;
        let right = (mono + reverbed.1) / 2.0;
        (left, right)
    }
}
