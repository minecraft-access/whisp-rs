#[cfg(target_os = "macos")]
pub mod av_speech_synthesizer;
pub mod espeak_ng;
#[cfg(windows)]
pub mod jaws;
#[cfg(windows)]
pub mod nvda;
#[cfg(windows)]
pub mod one_core;
#[cfg(windows)]
pub mod sapi;
#[cfg(target_os = "linux")]
pub mod speech_dispatcher;
use crate::audio::*;
use crate::error::OutputError;
use crate::metadata::*;
pub trait Backend {
  fn new() -> Result<Self, OutputError>
  where
    Self: Sized;
  fn name(&self) -> String;
  fn speech_metadata(&self) -> Option<SpeechSynthesizerMetadata> {
    match (
      self.as_speech_synthesizer_to_audio_data(),
      self.as_speech_synthesizer_to_audio_output(),
    ) {
      (None, None) => None,
      (Some(synthesizer), _) => Some(SpeechSynthesizerMetadata {
        name: self.name(),
        supports_speaking_to_audio_data: true,
        supports_speech_parameters: synthesizer.supports_speech_parameters(),
      }),
      (None, Some(synthesizer)) => Some(SpeechSynthesizerMetadata {
        name: self.name(),
        supports_speaking_to_audio_data: true,
        supports_speech_parameters: synthesizer.supports_speech_parameters(),
      }),
    }
  }
  fn braille_metadata(&self) -> Option<BrailleBackendMetadata> {
    self
      .as_braille_backend()
      .map(|backend| BrailleBackendMetadata {
        name: self.name(),
        priority: backend.priority(),
      })
  }
  fn list_voices(&self) -> Result<Vec<Voice>, OutputError>;
  fn as_speech_synthesizer_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData>;
  fn as_speech_synthesizer_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput>;
  fn as_braille_backend(&self) -> Option<&dyn BrailleBackend>;
}
pub trait SpeechSynthesizerToAudioData {
  fn supports_speech_parameters(&self) -> bool;
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> Result<SpeechResult, OutputError>;
}
pub trait SpeechSynthesizerToAudioOutput {
  fn supports_speech_parameters(&self) -> bool;
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
    interrupt: bool,
  ) -> Result<(), OutputError>;
  fn stop_speech(&self) -> Result<(), OutputError>;
}
pub trait BrailleBackend {
  fn priority(&self) -> u8;
  fn braille(&self, text: &str) -> Result<(), OutputError>;
}
