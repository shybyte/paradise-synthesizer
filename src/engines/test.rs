#![allow(dead_code)]

use midi_message::{MidiMessage, Note};
use soundpipe::factory::Factory;
use soundpipe::soundpipe::midi2cps;
use soundpipe::ugens::effects::revsc::Revsc;
use soundpipe::ugens::envelopes::adsr::Adsr;
use soundpipe::ugens::filter::wp_korg_35::WpKorg35;
use soundpipe::ugens::oscillators::bl_square::BlSquare;
use soundpipe::ugens::port::Port;
use soundpipe::Soundpipe;

use crate::pressed_notes::PressedNotes;
use crate::synth_engine::SynthEngine;
use crate::ugens::{FunctionOsc, UGenFactory};
use crate::unison::UnisonOscillator;

pub struct TestEngine {
    pressed_notes: PressedNotes,
    note: f32,
    port: Port,
    adsr: Adsr,
    sin_osc: FunctionOsc,
    osc1: UnisonOscillator,
    osc2: UnisonOscillator,
    sub_osc: BlSquare,
    filter: WpKorg35,
    reverb: Revsc,
    gate: f32,
}

impl TestEngine {
    pub fn new(sample_rate: u32) -> Self {
        let sp = Soundpipe::new(sample_rate as i32);
        let u_gen_factory = UGenFactory::new(sample_rate);

        let adsr = sp.adsr();
        adsr.set_attack_time(0.02);
        adsr.set_release_time(0.0);

        let osc1 = UnisonOscillator::new(&sp, 5, 0.1);
        let osc2 = UnisonOscillator::new(&sp, 5, 0.1);

        let reverb = sp.revsc();
        reverb.set_feedback(0.6);

        TestEngine {
            pressed_notes: PressedNotes::new(),
            note: 64.0,
            port: sp.port(0.02),
            adsr,
            sin_osc: u_gen_factory.sin(),
            // sin_osc: FunctionOsc::new(sample_rate, |x| x),
            osc1,
            osc2,
            sub_osc: sp.bl_square(),
            filter: sp.wpkorg35(),
            reverb,
            gate: 0.0,
        }
    }
}

impl TestEngine {
    fn set_note(&mut self, midi_note: Note) {
        self.note = midi_note as f32;
    }
}

impl SynthEngine for TestEngine {
    fn on_midi_message(&mut self, midi_message: MidiMessage) {
        match midi_message {
            MidiMessage::ControlChange(_, control, value) => {
                let normalized_value = value as f32 / 127.0;
                match control {
                    1 => self.sub_osc.set_width(normalized_value * 0.48 + 0.5),
                    74 => self.filter.set_cutoff(normalized_value * 10_000.0),
                    71 => self.filter.set_res(normalized_value * 2.0),
                    72 => self.filter.set_saturation(normalized_value * 5.0),
                    _ => {}
                }
            }
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
        self.osc1.set_note(smoothed_noted);
        self.osc2.set_note(smoothed_noted + 7.0);
        self.sub_osc.set_freq(midi2cps(self.note - 12.0));
        self.sin_osc.set_freq(midi2cps(self.note));

        let mix = self.sin_osc.compute();
        let mono = mix * self.adsr.compute(self.gate) * 0.7;
        (mono, mono)
    }
}
