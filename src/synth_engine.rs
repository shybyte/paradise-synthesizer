use midi_message::MidiMessage;

pub type StereoOutput = (f32, f32);

pub trait SynthEngine: Send {
    fn on_midi_message(&mut self, midi_message: MidiMessage);
    fn compute_output(&mut self) -> StereoOutput;
}
