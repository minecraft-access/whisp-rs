#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
include!(concat!(env!("OUT_DIR"), "/nvda_bindings.rs"));
use crate::speech_synthesizer::*;
use windows::core::*;
use windows::Win32::Foundation::WIN32_ERROR;
fn to_result(error: u32) -> Result<()> {
  WIN32_ERROR(error).ok()
}
pub struct Nvda;
impl SpeechSynthesizer for Nvda {
  fn new() -> std::result::Result<Self, SpeechError> {
    unsafe {
      to_result(nvdaController_initialize())?;
      to_result(nvdaController_testIfRunning())?;
      Ok(Nvda)
    }
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData {
      name: "NVDA".to_owned(),
      supports_to_audio_data: false,
      supports_to_audio_output: true,
      supports_speech_parameters: false,
    }
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, SpeechError> {
    Ok(vec![Voice {
      synthesizer: self.data(),
      display_name: "NVDA".to_owned(),
      name: "nvda".to_owned(),
      languages: vec![],
      priority: 0,
    }])
  }
  fn as_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    None
  }
  fn as_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    Some(self)
  }
}
impl SpeechSynthesizerToAudioOutput for Nvda {
  fn speak(
    &self,
    _voice: &str,
    _language: &str,
    _rate: Option<u8>,
    _volume: Option<u8>,
    _pitch: Option<u8>,
    text: &str,
    interrupt: bool,
  ) -> std::result::Result<(), SpeechError> {
    unsafe {
      if interrupt {
        to_result(nvdaController_cancelSpeech())?;
      };
      let text = HSTRING::from(text);
      to_result(nvdaController_speakText(text.as_ptr()))?;
      Ok(())
    }
  }
  fn stop_speech(&self) -> std::result::Result<(), SpeechError> {
    unsafe {
      to_result(nvdaController_cancelSpeech())?;
      Ok(())
    }
  }
}
