use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat};
// use std::fs::File;
// use std::io::BufWriter;
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
        .expect("No device supporting I24 found");

    println!("Device: {}", device.name()?);

    let config = device.default_input_config().unwrap();
    // let input_channels = config.channels();
    // let output_channels = 2.min(input_channels); // Only write first 2 channels
    // let spec = hound::WavSpec {
    //     channels: output_channels as _,
    //     sample_rate: config.sample_rate().0 as _,
    //     bits_per_sample: (config.sample_format().sample_size() * 8) as _,
    //     sample_format: match config.sample_format().is_float() {
    //         true => hound::SampleFormat::Float,
    //         false => hound::SampleFormat::Int,
    //     },
    // };

    // const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/recorded.wav");
    // let writer = hound::WavWriter::create(PATH, spec)?;
    // let writer = Arc::new(Mutex::new(Some(writer)));
    // let writer_2 = writer.clone();

    let err_fn = |e| println!("an error occurred on stream: {}", e);
    let timeout = std::time::Duration::from_secs(1);

    let last_end_pts = Arc::new(Mutex::new(None));

    let stream = device
        .build_input_stream(
            &config.config(),
            move |data: &[cpal::I24], info: &cpal::InputCallbackInfo| {
                write_input_sample::<cpal::I24, i32>(
                    data,
                    info,
                    config.channels(),
                    config.sample_rate().0,
                    last_end_pts.clone(),
                    // &writer_2,
                    // input_channels,
                    // output_channels,
                )
            },
            err_fn,
            Some(timeout),
        )
        .unwrap();

    stream.play()?;

    let record_duration = 100; // seconds
    println!("Recording {} seconds...", record_duration);
    std::thread::sleep(std::time::Duration::from_secs(record_duration));
    drop(stream);
    // writer.lock().unwrap().take().unwrap().finalize()?;
    // println!("Recording {} complete!", PATH);

    Ok(())
}

// type WavWriterHandle = Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>;

fn write_input_sample<T, U>(
    input: &[T],
    info: &cpal::InputCallbackInfo,
    channels: u16,
    sample_rate: u32,
    last_end_pts: Arc<Mutex<Option<cpal::StreamInstant>>>,
    // writer: &WavWriterHandle,
    // input_channels: u16,
    // output_channels: u16,
) where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    let sample_len = input.len();
    if sample_len > channels as usize * 1200 {
        println!("Bump of {} samples", sample_len);
    }

    let start_pts = info.timestamp().capture;
    let sample_duration_nanos = ((sample_len as u64) as f64 / channels as f64
        * 1_000_000_000.0
        / sample_rate as f64) as u64;
    let sample_duration = std::time::Duration::from_nanos(sample_duration_nanos);
    let end_pts = start_pts.add(sample_duration).unwrap();

    let last_end = { last_end_pts.lock().unwrap().clone() };
    if let Some(last_end) = last_end {
        let duration_since_last_frame = start_pts.duration_since(&last_end);
        println!("Duration since last frame: {duration_since_last_frame:?}");
    }

    println!("Processing {sample_len} samples from {start_pts:?} to {end_pts:?}");
    *last_end_pts.lock().unwrap() = Some(end_pts);

    // if let Ok(mut guard) = writer.try_lock() {
    //     if let Some(writer) = guard.as_mut() {
    //         // Process samples in chunks of input_channels
    //         for chunk in input.chunks(input_channels as usize) {
    //             // Only write the first output_channels samples from each chunk
    //             for (i, &sample) in chunk.iter().enumerate() {
    //                 if i < output_channels as usize {
    //                     let sample: U = U::from_sample(sample);
    //                     writer.write_sample(sample).ok();
    //                 }
    //             }
    //         }
    //     }
    // }
}
