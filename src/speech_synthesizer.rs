use std::error::Error;
use std::fmt;
pub struct Voice {
  pub synthesizer: String,
  pub display_name: String,
  pub name: String,
  pub language: String
}
#[derive(Clone,Debug)] #[repr(u8)] pub enum SampleFormat {
  S16 = 0,
  F32 = 1
}
#[derive(Debug)] pub struct SpeechResult {
  pub pcm: Vec<u8>,
  pub sample_format: SampleFormat,
  pub sample_rate: u32
}
#[derive(Debug)] pub struct SpeechError {
  pub message: String
}
impl fmt::Display for SpeechError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.message)
  }
}
impl<T: Error> From<T> for SpeechError {
  fn from(error: T) -> Self {
    SpeechError { message: error.to_string() }
  }
}
pub trait SpeechSynthesizer {
  fn new() -> Result<Self, SpeechError> where Self: Sized;
  fn name(&self) -> String;
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError>;
  fn speak(&self, voice: &str, language: &str, rate: u8, volume: u8, pitch: u8, text: &str) -> Result<SpeechResult, SpeechError>;
}
