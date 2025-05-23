use thiserror::Error;
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
