#![allow(non_snake_case)]
use crate::error::SpeechError;
use crate::speech_synthesizer::*;
use anyhow::anyhow;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
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
impl SpeechSynthesizer for Jaws {
  fn new() -> std::result::Result<Self, SpeechError> {
    unsafe {
      FindWindowW(w!("JFWUI2"), None).map_err(SpeechError::into_unknown)?;
      let guid = GUID::from_u128(0xCCE5B1E5_B2ED_45D5_B09F_8EC54B75ABF4);
      let jaws_api =
        CoCreateInstance(&guid, None, CLSCTX_ALL).map_err(SpeechError::into_unknown)?;
      Ok(Jaws { jaws_api })
    }
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData {
      name: "JAWS".to_owned(),
      supports_to_audio_data: false,
      supports_to_audio_output: true,
      supports_speech_parameters: false,
    }
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, SpeechError> {
    Ok(vec![Voice {
      synthesizer: self.data(),
      display_name: "JAWS".to_owned(),
      name: "jaws".to_owned(),
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
impl SpeechSynthesizerToAudioOutput for Jaws {
  fn speak(
    &self,
    _voice: Option<&str>,
    _language: Option<&str>,
    _rate: Option<u8>,
    _volume: Option<u8>,
    _pitch: Option<u8>,
    text: &str,
    interrupt: bool,
  ) -> std::result::Result<(), SpeechError> {
    let mut result: VARIANT_BOOL = Default::default();
    unsafe {
      self
        .jaws_api
        .SayString(text.into(), interrupt.into(), &mut result)
        .ok()
        .map_err(|err| SpeechError::into_speak_failed(&self.data().name, "jaws", err))?
    };
    if result.into() {
      Ok(())
    } else {
      Err(SpeechError::into_speak_failed(
        &self.data().name,
        "jaws",
        anyhow!("JAWS failed to speak"),
      ))
    }
  }
  fn stop_speech(&self) -> std::result::Result<(), SpeechError> {
    unsafe {
      self
        .jaws_api
        .StopSpeech()
        .ok()
        .map_err(|err| SpeechError::into_stop_speech_failed(&self.data().name, err))?
    };
    Ok(())
  }
}
