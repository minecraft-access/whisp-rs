use thiserror::Error;
#[derive(Debug, Error)]
pub enum OutputError {
  #[error("No backend has been registered with the name {0}")]
  BackendNotFound(String),
  #[error("The backend {0} does not support returning audio data")]
  AudioDataNotSupported(String),
  #[error("The backend {0} does not support speech")]
  SpeechNotSupported(String),
  #[error("The backend {0} does not support Braille")]
  BrailleNotSupported(String),
  #[error("No voice was found with the name {0}")]
  VoiceNotFound(String),
  #[error("No voice was found with the language {0}")]
  LanguageNotFound(String),
  #[error("No voices were found")]
  NoVoices,
  #[error("No Braille backends were found")]
  NoBrailleBackends,
  #[error("No output backends were found")]
  NoBackends,
  #[error("Speech rate ({0}) is not between 0 and 100")]
  InvalidRate(u8),
  #[error("Speech volume ({0}) is not between 0 and 100")]
  InvalidVolume(u8),
  #[error("Speech pitch ({0}) is not between 0 and 100")]
  InvalidPitch(u8),
  #[error("Failed to speak with the requested backend {backend} and voice {voice}: {error}")]
  SpeakFailed {
    backend: String,
    voice: String,
    error: anyhow::Error,
  },
  #[error("Failed to stop the requested backend {backend} from speaking: {error}")]
  StopSpeechFailed {
    backend: String,
    error: anyhow::Error,
  },
  #[error("Failed to Braille message with the requested backend {backend}: {error}")]
  BrailleFailed {
    backend: String,
    error: anyhow::Error,
  },
  #[error("Failed to initialize whisp-rs: {0}")]
  InitializeFailed(anyhow::Error),
  #[error("Unknown error: {0}")]
  Unknown(anyhow::Error),
}
impl OutputError {
  pub fn into_backend_not_found(backend: &str) -> Self {
    OutputError::BackendNotFound(backend.to_owned())
  }
  pub fn into_audio_data_not_supported(backend: &str) -> Self {
    OutputError::AudioDataNotSupported(backend.to_owned())
  }
  pub fn into_speech_not_supported(backend: &str) -> Self {
    OutputError::SpeechNotSupported(backend.to_owned())
  }
  pub fn into_braille_not_supported(backend: &str) -> Self {
    OutputError::BrailleNotSupported(backend.to_owned())
  }
  pub fn into_voice_not_found(voice: &str) -> Self {
    OutputError::VoiceNotFound(voice.to_owned())
  }
  pub fn into_language_not_found(language: &str) -> Self {
    OutputError::LanguageNotFound(language.to_owned())
  }
  pub fn into_speak_failed<T>(backend: &str, voice: &str, error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    OutputError::SpeakFailed {
      backend: backend.to_owned(),
      voice: voice.to_owned(),
      error: error.into(),
    }
  }
  pub fn into_stop_speech_failed<T>(backend: &str, error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    OutputError::StopSpeechFailed {
      backend: backend.to_owned(),
      error: error.into(),
    }
  }
  pub fn into_braille_failed<T>(backend: &str, error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    OutputError::BrailleFailed {
      backend: backend.to_owned(),
      error: error.into(),
    }
  }
  pub fn into_initialize_failed<T>(error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    OutputError::InitializeFailed(error.into())
  }
  pub fn into_unknown<T>(error: T) -> Self
  where
    T: Into<anyhow::Error>,
  {
    OutputError::Unknown(error.into())
  }
}
