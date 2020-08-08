extern crate anyhow;
extern crate cpal;
extern crate midi_message;
extern crate midir;

use std::sync::mpsc;

use cpal::{Device, SupportedStreamConfig};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use midi_message::MidiMessage;
use midir::{Ignore, MidiInput};

use crate::synth_engine::SynthEngine;
use crate::young::Young;

pub mod synth_engine;
pub mod young;

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

    for port in in_ports.iter() {
        eprintln!("M = {:?}", midi_in.port_name(&port));
    }

    let in_port = in_ports
        .iter()
        .find(|it| {
            let name = midi_in.port_name(it).unwrap();
            name.contains("VMPK") || name.contains("K-Board")
        })
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

    let channels = config.channels as usize;

    eprintln!("sample_rate = {:?}", config.sample_rate.0);
    eprintln!("channels = {:?}", channels);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let mut synth_engine = Young::new(config.sample_rate.0);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            while let Ok(midi_message) = midi_rx.try_recv() {
                eprintln!(
                    "midi_message = {:?}, data.len = {:?}",
                    midi_message,
                    data.len()
                );
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
