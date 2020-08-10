use midi_message::{MidiMessage, Note};
use soundpipe::factory::Factory;
use soundpipe::Soundpipe;
use soundpipe::soundpipe::midi2cps;
use soundpipe::ugens::effects::revsc::Revsc;
use soundpipe::ugens::envelopes::adsr::Adsr;
use soundpipe::ugens::oscillators::bl_saw::BlSaw;
use soundpipe::ugens::oscillators::common::MonoOsc;
use soundpipe::ugens::port::Port;

use crate::pressed_notes::PressedNotes;
use crate::synth_engine::SynthEngine;

pub struct Young {
    pressed_notes: PressedNotes,
    note: f32,
    port: Port,
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

        let port = sp.port(0.02);

        Young {
            pressed_notes: PressedNotes::new(),
            note: 64.0,
            port,
            adsr,
            osc1,
            osc2,
            reverb,
            gate: 0.0,
        }
    }
}

impl Young {
    fn set_note(&mut self, midi_note: Note) {
        self.note = midi_note as f32;
    }
}

impl SynthEngine for Young {
    fn on_midi_message(&mut self, midi_message: MidiMessage) {
        match midi_message {
            MidiMessage::NoteOn(_, midi_note, _) => {
                self.pressed_notes.note_on(midi_note);
                self.set_note(midi_note);
                self.gate = 1.0;
            }
            MidiMessage::NoteOff(_, midi_note, _) => {
                self.pressed_notes.note_off(midi_note);
                if let Some(remaining_note) = self.pressed_notes.get_current_note() {
                    self.set_note(remaining_note);
                } else {
                    self.gate = 0.0;
                }
            }
            _ => {}
        }
    }

    fn compute_output(&mut self) -> (f32, f32) {
        let smoothed_noted = self.port.compute(self.note);
        self.osc1.set_freq(midi2cps(smoothed_noted));
        self.osc2.set_freq(midi2cps(smoothed_noted + 7.0));
        let mix = (self.osc1.compute() + self.osc2.compute()) / 2.0;
        let mono = mix * self.adsr.compute(self.gate) * 0.7;
        let reverbed = self.reverb.compute(mono, mono);
        let left = (mono + reverbed.0) / 2.0;
        let right = (mono + reverbed.1) / 2.0;
        (left, right)
    }
}
