#![allow(dead_code)]

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    let device = host
        .input_devices()?
        .find(|d| {
            d.supported_input_configs()
                .map(|mut configs| {
                    configs.any(|c| c.sample_format() == SampleFormat::I24)
                })
                .unwrap_or(false)
        })
        .expect("No input device supporting I24 found");

    if !device
        .supported_output_configs()
        .map(|mut configs| configs.any(|c| c.sample_format() == SampleFormat::I24))
        .unwrap_or(false)
    {
        return Err("No output device supporting I24 found".into());
    }

    println!("Device: {}", device.name()?);

    let input_config = device.default_input_config().unwrap();
    let output_config = device.default_output_config().unwrap();

    println!("Input config: {:?}", input_config);
    println!("Output config: {:?}", output_config);

    let input_channels = input_config.channels();
    let output_channels = output_config.channels();

    // Simple ring buffer using VecDeque
    let buffer_size = (input_config.sample_rate().0 as usize) * 2;
    let ring_buffer =
        Arc::new(Mutex::new(VecDeque::<cpal::I24>::with_capacity(buffer_size)));
    let ring_buffer_input = ring_buffer.clone();
    let ring_buffer_output = ring_buffer.clone();

    // Pre-fill buffer with silence to add some latency
    let latency_samples = (input_config.sample_rate().0 as usize) / 100; // 10ms latency
    {
        let mut buffer = ring_buffer.lock().unwrap();
        for _ in 0..latency_samples {
            buffer.push_back(cpal::I24::EQUILIBRIUM);
        }
    }

    let err_fn = |e| println!("an error occurred on stream: {}", e);
    let timeout = std::time::Duration::from_secs(1);

    let input_stream = device
        .build_input_stream(
            &input_config.config(),
            move |data: &[cpal::I24], _: &_| {
                let mut buffer = ring_buffer_input.lock().unwrap();
                for chunk in data.chunks(input_channels as usize) {
                    if let Some(first_sample) = chunk.first() {
                        buffer.push_back(*first_sample);
                        // Keep buffer size reasonable
                        if buffer.len() > buffer_size {
                            buffer.pop_front();
                        }
                    }
                }
            },
            err_fn,
            Some(timeout),
        )
        .unwrap();

    let output_stream = device
        .build_output_stream(
            &output_config.config(),
            move |data: &mut [cpal::I24], _: &_| {
                let mut buffer = ring_buffer_output.lock().unwrap();
                for chunk in data.chunks_mut(output_channels as usize) {
                    if let Some(sample) = buffer.pop_front() {
                        // Write sample to first channel
                        chunk[0] = sample;
                        // Fill other channels with silence
                        for channel in chunk.iter_mut().skip(1) {
                            *channel = cpal::I24::EQUILIBRIUM;
                        }
                    } else {
                        // No data available, fill all channels with silence
                        for channel in chunk.iter_mut() {
                            *channel = cpal::I24::EQUILIBRIUM;
                        }
                    }
                }
            },
            err_fn,
            Some(timeout),
        )
        .unwrap();

    input_stream.play()?;
    output_stream.play()?;

    println!("Piping first input channel to first output channel...");
    println!("Press Ctrl+C to stop");

    std::thread::sleep(std::time::Duration::from_secs(3600));

    Ok(())
}
