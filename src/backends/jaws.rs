#![allow(non_snake_case)]
use crate::backends::{
  Backend, BrailleBackend, SpeechSynthesizerToAudioData, SpeechSynthesizerToAudioOutput,
};
use crate::error::OutputError;
use crate::metadata::Voice;
use anyhow::anyhow;
use windows::core::{interface, w, BSTR, GUID, HRESULT};
use windows::Win32::Foundation::VARIANT_BOOL;
use windows::Win32::System::Com::{
  CoCreateInstance, IDispatch, IDispatch_Impl, IDispatch_Vtbl, CLSCTX_ALL,
};
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
#[interface("123DEDB4-2CF6-429C-A2AB-CC809E5516CE")]
unsafe trait IJawsApi: IDispatch {
  fn RunScript(&self, ScriptName: BSTR, vbSuccess: *mut VARIANT_BOOL) -> HRESULT;
  fn SayString(
    &self,
    StringToSpeak: BSTR,
    bFlush: VARIANT_BOOL,
    vbSuccess: *mut VARIANT_BOOL,
  ) -> HRESULT;
  fn StopSpeech(&self) -> HRESULT;
  fn Enable(&self, vbNoDDIHooks: VARIANT_BOOL, vbSuccess: *mut VARIANT_BOOL) -> HRESULT;
  fn Disable(&self, vbSuccess: *mut VARIANT_BOOL) -> HRESULT;
  fn RunFunction(&self, FunctionName: BSTR, vbSuccess: *mut VARIANT_BOOL) -> HRESULT;
}
pub struct Jaws {
  jaws_api: IJawsApi,
}
impl Backend for Jaws {
  fn new() -> std::result::Result<Self, OutputError> {
    unsafe {
      FindWindowW(w!("JFWUI2"), None).map_err(OutputError::into_unknown)?;
      let guid = GUID::from_u128(0xCCE5B1E5_B2ED_45D5_B09F_8EC54B75ABF4);
      let jaws_api =
        CoCreateInstance(&guid, None, CLSCTX_ALL).map_err(OutputError::into_unknown)?;
      Ok(Jaws { jaws_api })
    }
  }
  fn name(&self) -> String {
    "JAWS".to_owned()
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, OutputError> {
    Ok(vec![Voice {
      synthesizer: self.speech_metadata().unwrap(),
      display_name: "JAWS".to_owned(),
      name: "jaws".to_owned(),
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
    Some(self)
  }
}
impl SpeechSynthesizerToAudioOutput for Jaws {
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
    let mut result: VARIANT_BOOL = Default::default();
    unsafe {
      self
        .jaws_api
        .SayString(text.into(), interrupt.into(), &mut result)
        .ok()
        .map_err(|err| OutputError::into_speak_failed(&self.name(), "jaws", err))?;
    };
    if result.into() {
      Ok(())
    } else {
      Err(OutputError::into_speak_failed(
        &self.name(),
        "jaws",
        anyhow!("JAWS failed to speak"),
      ))
    }
  }
  fn stop_speech(&self) -> std::result::Result<(), OutputError> {
    unsafe {
      self
        .jaws_api
        .StopSpeech()
        .ok()
        .map_err(|err| OutputError::into_stop_speech_failed(&self.name(), err))?;
    };
    Ok(())
  }
}
impl BrailleBackend for Jaws {
  fn priority(&self) -> u8 {
    0
  }
  fn braille(&self, text: &str) -> std::result::Result<(), OutputError> {
    let function = "BrailleString(\"".to_owned() + &text.replace('"', "'") + "\")";
    let mut result: VARIANT_BOOL = Default::default();
    unsafe {
      self
        .jaws_api
        .RunFunction(function.into(), &mut result)
        .ok()
        .map_err(|err| OutputError::into_braille_failed(&self.name(), err))?;
    };
    if result.into() {
      Ok(())
    } else {
      Err(OutputError::into_braille_failed(
        &self.name(),
        anyhow!("JAWS failed to Braille the message"),
      ))
    }
  }
}
