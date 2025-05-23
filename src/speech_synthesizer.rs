use thiserror::Error;
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
#[derive(Debug, Error)]
pub enum SpeechError {
  #[error("No synthesizer has been registered with the name {0}")]
  SynthesizerNotFound(String),
  #[error("The synthesizer {0} does not support returning audio data")]
  AudioDataNotSupported(String),
  #[error("The synthesizer {0} does not support speech")]
  SpeechNotSupported(String),
  #[error("No voice was found with the name {0}")]
  VoiceNotFound(String),
  #[error("No voice was found with the language {0}")]
  LanguageNotFound(String),
  #[error("No voices were found")]
  NoVoices,
  #[error("Speech rate ({0}) is not between 0 and 100")]
  InvalidRate(u8),
  #[error("Speech volume ({0}) is not between 0 and 100")]
  InvalidVolume(u8),
  #[error("Speech pitch ({0}) is not between 0 and 100")]
  InvalidPitch(u8),
  #[error(
    "Failed to speak with the requested synthesizer ({synthesizer} and voice {voice}: {error}"
  )]
  SpeakFailed {
    synthesizer: String,
    voice: String,
    error: anyhow::Error,
  },
  #[error("Failed to stop the requested synthesizer ({synthesizer} from speaking: {error}")]
  StopSpeechFailed {
    synthesizer: String,
    error: anyhow::Error,
  },
  #[error("Playing audio with Rodio failed: {0}")]
  PlayAudioFailed(anyhow::Error),
  #[error("Stopping audio with Rodio failed: {0}")]
  StopAudioFailed(anyhow::Error),
  #[error("Failed to initialize whisp-rs: {0}")]
  InitializeFailed(anyhow::Error),
  #[error("Unknown error: {0}")]
  Unknown(anyhow::Error),
}
impl SpeechError {
  pub fn into_synthesizer_not_found(synthesizer: &str) -> Self {
    SpeechError::SynthesizerNotFound(synthesizer.to_owned())
  }
  pub fn into_audio_data_not_supported(synthesizer: &str) -> Self {
    SpeechError::AudioDataNotSupported(synthesizer.to_owned())
  }
  pub fn into_speech_not_supported(synthesizer: &str) -> Self {
    SpeechError::SpeechNotSupported(synthesizer.to_owned())
  }
  pub fn into_voice_not_found(voice: &str) -> Self {
    SpeechError::VoiceNotFound(voice.to_owned())
  }
  pub fn into_language_not_found(language: &str) -> Self {
    SpeechError::LanguageNotFound(language.to_owned())
  }
  pub fn into_speak_failed<T>(synthesizer: &str, voice: &str, error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    SpeechError::SpeakFailed {
      synthesizer: synthesizer.to_owned(),
      voice: voice.to_owned(),
      error: error.into(),
    }
  }
  pub fn into_stop_speech_failed<T>(synthesizer: &str, error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    SpeechError::StopSpeechFailed {
      synthesizer: synthesizer.to_owned(),
      error: error.into(),
    }
  }
  pub fn into_play_audio_failed<T>(error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    SpeechError::PlayAudioFailed(error.into())
  }
  pub fn into_stop_audio_failed<T>(error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    SpeechError::StopAudioFailed(error.into())
  }
  pub fn into_initialize_failed<T>(error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    SpeechError::InitializeFailed(error.into())
  }
  pub fn into_unknown<T>(error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    SpeechError::Unknown(error.into())
  }
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
