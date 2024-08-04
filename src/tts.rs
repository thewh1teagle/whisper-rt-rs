use std::time::Duration;
use std::{
    sync::Arc, 
    error::Error
};

use once_cell::sync::Lazy;

use piper::{
    synth::PiperSpeechSynthesizer,
    vits::VitsModel
};
use rodio::{Decoder, OutputStream, source::Source, Sink};
use std::fs::File;
use std::io::BufReader;

static ENVIRONMENT: Lazy<Arc<ort::Environment>> = Lazy::new(|| Arc::new(ort::Environment::default()));

pub struct Tts {
    synthesizer: PiperSpeechSynthesizer
}

impl Tts {
    pub fn new() -> Self {
        let speaker = Arc::new(VitsModel::new("en_GB-alba-medium.onnx.json".into(), "en_GB-alba-medium.onnx".into(), &ENVIRONMENT).unwrap());
        // let speaker = Arc::new(VitsModel::new("uk_UA-ukrainian_tts-medium.onnx.json".into(), "uk_UA-ukrainian_tts-medium.onnx".into(), &ENVIRONMENT)?);
        // for speaker in speaker.speakers()? {
        //     println!("Speaker {}: {}", speaker.0, speaker.1);
        // }
        speaker.set_length_scale(1.).unwrap();
        let synthesizer = PiperSpeechSynthesizer::new(speaker).unwrap();
        Self {synthesizer}

    }

    pub fn speak(&mut self, prompt: String) {
        self.synthesizer.synthesize_to_wav_file("out.wav", prompt).unwrap();
    
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open("out.wav").unwrap());
        let source = Decoder::new(file).unwrap();
    
        // Create a sink
        let sink = Sink::try_new(&stream_handle).unwrap();
    
        // Add the source to the sink
        sink.append(source);
    
        // Sleep until the sound ends
        sink.sleep_until_end();
    }
}