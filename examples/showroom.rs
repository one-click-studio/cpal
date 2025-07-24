use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();

    let device = host
        .input_devices()?
        .find(|d| d.name().unwrap().contains("XR18"))
        .expect("No device supporting I24 found");

    println!("Device: {}", device.name()?);

    let config = device.default_input_config().unwrap();

    let err_fn = |e| println!("an error occurred on stream: {}", e);
    let timeout = std::time::Duration::from_secs(1);

    let last_end_pts = Arc::new(Mutex::new(None));

    let stream = match config.sample_format() {
        SampleFormat::I8 => device
            .build_input_stream(
                &config.config(),
                move |data: &[i8], info: &cpal::InputCallbackInfo| {
                    write_input_sample(
                        data.len(),
                        info,
                        config.channels(),
                        config.sample_rate().0,
                        last_end_pts.clone(),
                    )
                },
                err_fn,
                Some(timeout),
            )
            .unwrap(),
        SampleFormat::I16 => device
            .build_input_stream(
                &config.config(),
                move |data: &[i16], info: &cpal::InputCallbackInfo| {
                    write_input_sample(
                        data.len(),
                        info,
                        config.channels(),
                        config.sample_rate().0,
                        last_end_pts.clone(),
                    )
                },
                err_fn,
                Some(timeout),
            )
            .unwrap(),
        SampleFormat::I32 => device
            .build_input_stream(
                &config.config(),
                move |data: &[i32], info: &cpal::InputCallbackInfo| {
                    write_input_sample(
                        data.len(),
                        info,
                        config.channels(),
                        config.sample_rate().0,
                        last_end_pts.clone(),
                    )
                },
                err_fn,
                Some(timeout),
            )
            .unwrap(),
        SampleFormat::F32 => device
            .build_input_stream(
                &config.config(),
                move |data: &[f32], info: &cpal::InputCallbackInfo| {
                    write_input_sample(
                        data.len(),
                        info,
                        config.channels(),
                        config.sample_rate().0,
                        last_end_pts.clone(),
                    )
                },
                err_fn,
                Some(timeout),
            )
            .unwrap(),
        SampleFormat::F64 => device
            .build_input_stream(
                &config.config(),
                move |data: &[f64], info: &cpal::InputCallbackInfo| {
                    write_input_sample(
                        data.len(),
                        info,
                        config.channels(),
                        config.sample_rate().0,
                        last_end_pts.clone(),
                    )
                },
                err_fn,
                Some(timeout),
            )
            .unwrap(),
        SampleFormat::I24 => device
            .build_input_stream(
                &config.config(),
                move |data: &[cpal::I24], info: &cpal::InputCallbackInfo| {
                    write_input_sample(
                        data.len(),
                        info,
                        config.channels(),
                        config.sample_rate().0,
                        last_end_pts.clone(),
                    )
                },
                err_fn,
                Some(timeout),
            )
            .unwrap(),

        _ => panic!("Unsupported sample format: {:?}", config.sample_format()),
    };

    stream.play()?;

    let record_duration = 100; // seconds
    println!("Recording {} seconds...", record_duration);
    std::thread::sleep(std::time::Duration::from_secs(record_duration));
    drop(stream);

    Ok(())
}

fn write_input_sample(
    sample_len: usize,
    info: &cpal::InputCallbackInfo,
    channels: u16,
    sample_rate: u32,
    last_end_pts: Arc<Mutex<Option<cpal::StreamInstant>>>,
) {
    static START: OnceCell<Instant> = OnceCell::new();
    let start_time = START.get_or_init(|| Instant::now());

    if sample_len > channels as usize * 1200 {
        println!("WARN: Bump of {} samples", sample_len);
    }

    let start_pts = info.timestamp().capture;
    let sample_duration_nanos = ((sample_len as u64) as f64 / channels as f64 * 1_000_000_000.0
        / sample_rate as f64) as u64;
    let sample_duration = std::time::Duration::from_nanos(sample_duration_nanos);
    let end_pts = start_pts.add(sample_duration).unwrap();

    let last_end = { last_end_pts.lock().unwrap().clone() };
    if let Some(last_end) = last_end {
        if let Some(duration_since_last_frame) = start_pts.duration_since(&last_end) {
            let duration_since_last_frame_nanos = duration_since_last_frame.as_nanos();
            let sample_duration_nanos = (1_000_000_000 / sample_rate) as u128;
            if duration_since_last_frame_nanos > sample_duration_nanos / 2 {
                println!("WARN: Duration since last frame: {duration_since_last_frame:?}. Elapsed time: {:?}", start_time.elapsed());
            }
        }
    }

    // println!("Processing {sample_len} samples from {start_pts:?} to {end_pts:?}");
    *last_end_pts.lock().unwrap() = Some(end_pts);
}
