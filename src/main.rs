use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eyre::{bail, Result};
use fon::chan::Ch32;
use fon::Audio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use webrtc_vad::Vad;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};
use circular_buffer::CircularBuffer;

struct VadWithSend(Vad);
unsafe impl Send for VadWithSend {}

type VadHandle = Arc<Mutex<VadWithSend>>;
type TimestampHandle = Arc<Mutex<Instant>>;
type SpeechStateHandle = Arc<Mutex<bool>>;
type WhisperHandle = Arc<Mutex<Whisper>>;

struct Whisper {
    state: WhisperState,
    params: FullParams<'static, 'static>,
}

impl Whisper {
    pub fn new() -> Self {
        whisper_rs::install_whisper_tracing_trampoline();
        let ctx_params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params("ggml-tiny.bin", ctx_params).unwrap();
        let state = ctx.create_state().unwrap();
        let mut params = FullParams::new(SamplingStrategy::default());
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_single_segment(true);
        params.set_debug_mode(false);
        
        Self { state, params }
    }

    pub fn transcribe(&mut self, samples: &[f32]) -> String {
        let samples: Vec<i16>= samples.iter().map(|&s| s as i16).collect();
        let mut new_samples = vec![0.0f32; samples.len()];
        whisper_rs::convert_integer_to_float_audio(&samples, &mut new_samples).unwrap();
        self.state.full(self.params.clone(), &new_samples).unwrap();
        self.state.full(self.params.clone(), &new_samples).unwrap();
        let num_segments = self.state.full_n_segments().unwrap();
        let mut text = String::new();
        for s in 0..num_segments {
            text += &self.state.full_get_segment_text_lossy(s).unwrap();
        }
        text
    }
}

fn main() -> Result<()> {
    let mut buf = CircularBuffer::<960000, f32>::new(); // 60s (16000 * 60)
    let mut whisper = Whisper::new();
    let whisper_handle = Arc::new(Mutex::new(whisper));
    let host = cpal::default_host();

    // Set up the input device and stream with the default input config.
    let device = host.default_input_device().unwrap();

    println!("Input device: {}", device.name()?);

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    println!("Default input config: {:?}", config);

    let mut vad = Vad::new();
    vad.set_mode(webrtc_vad::VadMode::VeryAggressive);
    let vad = Arc::new(Mutex::new(VadWithSend(vad)));

    // A flag to indicate that recording is in progress.
    println!("Begin recording...");

    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };

    let last_speech_timestamp = Arc::new(Mutex::new(Instant::now()));
    let is_speeching = Arc::new(Mutex::new(false));

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            let vad_clone = Arc::clone(&vad);
            let last_speech_timestamp_clone = Arc::clone(&last_speech_timestamp);
            let is_speeching_clone = Arc::clone(&is_speeching);
            device.build_input_stream(
                &config.into(),
                move |data, _: &_| handle_input_data(data, &vad_clone, &last_speech_timestamp_clone, &is_speeching_clone),
                err_fn,
                None,
            )?
        },
        sample_format => {
            bail!(
                "Unsupported sample format '{sample_format}'"
            )
        }
    };

    stream.play()?;

    // Let recording go for roughly ten seconds.
    std::thread::sleep(std::time::Duration::from_secs(10));
    drop(stream);
    Ok(())
}

fn handle_input_data(input: &[f32], vad: &VadHandle, last_speech_timestamp: &TimestampHandle, is_speeching: &SpeechStateHandle) {
    let mut vad = vad.lock().unwrap();
    let mut last_speech_timestamp = last_speech_timestamp.lock().unwrap();
    let mut is_speeching = is_speeching.lock().unwrap();

    let audio = Audio::<Ch32, 2>::with_f32_buffer(48000, input);
    let mut audio = Audio::<Ch32, 2>::with_audio(16000, &audio);
    let resampled = audio.as_f32_slice();
    // volume up a bit
    let resampled: Vec<f32> = audio.as_f32_slice().iter().map(|&x| x * 5.0).collect();
    let mut i16_chunk: Vec<i16> = resampled.iter().map(|&x| (x * 32767.0) as i16).collect();
    i16_chunk.truncate(10 * 16000 / 1000);
    let is_speech = vad.0.is_voice_segment(&i16_chunk).unwrap_or_default();

    if is_speech {
        *last_speech_timestamp = Instant::now();
        if !*is_speeching {
            *is_speeching = true;
            println!("Speech detected.");
        }
    } else {
        if *is_speeching && last_speech_timestamp.elapsed() > Duration::from_millis(900) {
            *is_speeching = false;
            println!("End of speech detected.");
        }
    }
}
