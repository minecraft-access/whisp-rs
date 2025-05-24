#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
include!(concat!(env!("OUT_DIR"), "/nvda_bindings.rs"));
use crate::backends::*;
use crate::error::OutputError;
use crate::metadata::Voice;
use windows::core::*;
use windows::Win32::Foundation::WIN32_ERROR;
fn to_result(error: u32) -> Result<()> {
  WIN32_ERROR(error).ok()
}
pub struct Nvda;
impl Backend for Nvda {
  fn new() -> std::result::Result<Self, OutputError> {
    unsafe {
      to_result(nvdaController_initialize()).map_err(OutputError::into_unknown)?;
      to_result(nvdaController_testIfRunning()).map_err(OutputError::into_unknown)?;
      Ok(Nvda)
    }
  }
  fn name(&self) -> String {
    "NVDA".to_owned()
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, OutputError> {
    Ok(vec![Voice {
      synthesizer: self.speech_metadata().unwrap(),
      display_name: "NVDA".to_owned(),
      name: "nvda".to_owned(),
      languages: vec![],
      priority: 0,
    }])
  }
  fn as_speech_synthesizer_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    None
  }
  fn as_speech_synthesizer_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    Some(self)
  }
  fn as_braille_backend(&self) -> Option<&dyn BrailleBackend> {
    None
  }
}
impl SpeechSynthesizerToAudioOutput for Nvda {
  fn supports_speech_parameters(&self) -> bool {
    false
  }
  fn speak(
    &self,
    _voice: Option<&str>,
    _language: Option<&str>,
    _rate: Option<u8>,
    _volume: Option<u8>,
    _pitch: Option<u8>,
    text: &str,
    interrupt: bool,
  ) -> std::result::Result<(), OutputError> {
    unsafe {
      if interrupt {
        to_result(nvdaController_cancelSpeech())
          .map_err(|err| OutputError::into_stop_speech_failed(&self.name(), err))?;
      };
      let text = HSTRING::from(text);
      to_result(nvdaController_speakText(text.as_ptr()))
        .map_err(|err| OutputError::into_speak_failed(&self.name(), "nvda", err))?;
      Ok(())
    }
  }
  fn stop_speech(&self) -> std::result::Result<(), OutputError> {
    unsafe {
      to_result(nvdaController_cancelSpeech())
        .map_err(|err| OutputError::into_stop_speech_failed(&self.name(), err))?;
      Ok(())
    }
  }
}
