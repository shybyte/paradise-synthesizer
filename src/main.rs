extern crate anyhow;
extern crate cpal;
extern crate midi_message;
extern crate midir;

use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SupportedStreamConfig};
use midi_message::MidiMessage;
use midir::{Ignore, MidiInput};

use soundpipe::soundpipe::midi2cps;
use soundpipe::Soundpipe;
use soundpipe::SoundpipeFactory;

struct SynthState {
    gate: f32,
}

fn main() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let device: Device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config: SupportedStreamConfig = device.default_output_config()?;

    eprintln!("config.s = {:?}", config.sample_format());

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into())?,
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into())?,
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into())?,
    }

    Ok(())
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
    where
        T: cpal::Sample,
{
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    let in_port = in_ports
        .iter()
        .find(|it| midi_in.port_name(it).unwrap().contains("VMPK"))
        .unwrap();

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let (tx, rx) = mpsc::channel();
    let (midi_tx, midi_rx) = mpsc::channel();

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in
        .connect(
            in_port,
            "midir-read-input",
            move |stamp, message, _| {
                println!("{}: {:?} (len = {})", stamp, message, message.len());
                midi_tx
                    .send(MidiMessage::new(message[0], message[1], message[2]))
                    .unwrap();
                if message[1] == 49 {
                    tx.send(true).unwrap();
                }
            },
            (),
        )
        .unwrap();

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    eprintln!("sample_rate = {:?}", sample_rate);
    eprintln!("channels = {:?}", channels);

    let sp = Soundpipe::new(sample_rate as i32);
    let adsr = sp.adsr();
    adsr.set_attack_time(0.01);
    let bl_saw = sp.bl_saw();
    let bl_saw2 = sp.bl_saw();
    let revsc = sp.revsc();
    revsc.set_feedback(0.6);

    bl_saw.set_freq(220.2);
    bl_saw2.set_freq(110.0);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let mut synth_state = SynthState { gate: 0.0 };

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            if let Ok(midi_message) = midi_rx.try_recv() {
                eprintln!(
                    "midi_message = {:?}, data.len = {:?}",
                    midi_message,
                    data.len()
                );
                match midi_message {
                    MidiMessage::NoteOn(_, midi_note, _) => {
                        let freq = midi2cps(midi_note as f32);
                        bl_saw.set_freq(freq);
                        bl_saw2.set_freq(freq);
                        synth_state.gate = 1.0;
                    }
                    MidiMessage::NoteOff(_, _, _) => {
                        synth_state.gate = 0.0;
                    }
                    _ => {}
                }
            }
            for frame in data.chunks_mut(channels) {
                let mono = (bl_saw.compute() + bl_saw2.compute()) / 2.0
                    * adsr.compute(synth_state.gate)
                    * 0.7;
                let reverbed = revsc.compute(mono, mono);
                let left = (mono + reverbed.0) / 2.0;
                let right = (mono + reverbed.1) / 2.0;
                frame[0] = cpal::Sample::from::<f32>(&left);
                frame[1] = cpal::Sample::from::<f32>(&right);
            }
        },
        err_fn,
    )?;
    stream.play()?;

    rx.recv().unwrap();
    println!("Closing connection");

    Ok(())
}
