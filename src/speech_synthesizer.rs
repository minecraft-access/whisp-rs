use crate::error::SpeechError;
#[derive(Debug)]
pub struct SpeechSynthesizerData {
  pub name: String,
  pub supports_to_audio_data: bool,
  pub supports_to_audio_output: bool,
  pub supports_speech_parameters: bool,
}
#[derive(Debug)]
pub struct Voice {
  pub synthesizer: SpeechSynthesizerData,
  pub display_name: String,
  pub name: String,
  pub languages: Vec<String>,
  pub priority: u8,
}
#[derive(Clone, Debug)]
#[repr(u8)]
pub enum SampleFormat {
  S16 = 0,
  F32 = 1,
}
#[derive(Debug)]
pub struct SpeechResult {
  pub pcm: Vec<u8>,
  pub sample_format: SampleFormat,
  pub sample_rate: u32,
}
pub trait SpeechSynthesizer {
  fn new() -> Result<Self, SpeechError>
  where
    Self: Sized;
  fn data(&self) -> SpeechSynthesizerData;
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError>;
  fn as_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData>;
  fn as_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput>;
}
pub trait SpeechSynthesizerToAudioData {
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> Result<SpeechResult, SpeechError>;
}
pub trait SpeechSynthesizerToAudioOutput {
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
    interrupt: bool,
  ) -> Result<(), SpeechError>;
  fn stop_speech(&self) -> Result<(), SpeechError>;
}
