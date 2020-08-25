extern crate anyhow;
extern crate cpal;
extern crate midi_message;
extern crate midir;

use std::sync::mpsc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Device, StreamConfig, SupportedStreamConfig};
use midi_message::MidiMessage;
use midir::{Ignore, MidiInput, MidiInputPort};

use crate::engines::create_enginge;

mod engines;
mod pressed_notes;
mod synth_engine;
mod unison;

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
    let midi_in = MidiInput::new("paradise-synth")?;

    let in_ports = midi_in.ports();

    let in_ports_with_names = in_ports
        .into_iter()
        .map(|port| (midi_in.port_name(&port).unwrap(), port))
        .collect::<Vec<_>>();

    for (name, _) in &in_ports_with_names {
        eprintln!("MidiInput = {:?}", name);
    }

    let (in_port_name, in_port) = find_wanted_input_port(in_ports_with_names).unwrap();

    println!("\nOpening connection");

    let (tx, rx) = mpsc::channel();
    let (midi_tx, midi_rx) = mpsc::channel();

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in
        .connect(
            &in_port,
            "midir-read-input",
            move |_stamp, message, _| {
                // println!("{}: {:?} (len = {})", stamp, message, message.len());
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

    let sample_rate = config.sample_rate.0;
    let channels = config.channels as usize;

    eprintln!("sample_rate = {:?}", sample_rate);
    eprintln!("channels = {:?}", channels);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let mut synth_engine = create_enginge(1, sample_rate);

    let low_latency_config = StreamConfig {
        buffer_size: BufferSize::Fixed(2048),
        channels: config.channels,
        sample_rate: config.sample_rate,
    };

    let stream = device.build_output_stream(
        &low_latency_config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Ok(midi_message) = midi_rx.try_recv() {
                if let MidiMessage::ProgramChange(_, program) = midi_message {
                    eprintln!("program = {:?}", program);
                    synth_engine = create_enginge(program, sample_rate);
                }
                // eprintln!("midi_message = {:?}", midi_message,);
                synth_engine.on_midi_message(midi_message);
            }

            for frame in data.chunks_mut(channels) {
                let (left, right) = synth_engine.compute_output();
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

fn find_wanted_input_port(
    mut input_ports: Vec<(String, MidiInputPort)>,
) -> Option<(String, MidiInputPort)> {
    let wanted_names = vec!["VMPK", "K-Board", "Through"];
    for wanted_name in wanted_names {
        let matching_input_port = input_ports
            .iter()
            .position(|(port_name, _)| port_name.contains(wanted_name));
        if let Some(input_port_index) = matching_input_port {
            return Some(input_ports.swap_remove(input_port_index));
        }
    }
    None
}
