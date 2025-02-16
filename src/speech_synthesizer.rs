use std::error::Error;
use std::fmt;
pub struct SpeechResult {
  pub pcm: Vec<u8>,
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
  fn min_rate(&self) -> u32;
  fn max_rate(&self) -> u32;
  fn list_voices(&self, language: &str) -> Result<Vec<String>, SpeechError>;
  fn speak(&self, voice: &str, rate: u32, volume: u8, pitch: u8, pitch_range: u8, text: &str) -> Result<SpeechResult, SpeechError>;
}
