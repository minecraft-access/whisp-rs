use crate::audio::{SampleFormat, SpeechResult};
use crate::metadata::{BrailleBackendMetadata, SpeechSynthesizerMetadata, Voice};
use std::ffi::{c_char, c_uchar, c_uint, CString};
#[repr(C)]
pub struct WhisprsSpeechResult {
  pub pcm: *mut u8,
  pub pcm_len: usize,
  pub sample_format: SampleFormat,
  pub sample_rate: c_uint,
}
impl From<SpeechResult> for WhisprsSpeechResult {
  fn from(result: SpeechResult) -> Self {
    let pcm_len = result.pcm.len();
    let mut pcm_box = result.pcm.into_boxed_slice();
    let pcm = pcm_box.as_mut_ptr();
    std::mem::forget(pcm_box);
    WhisprsSpeechResult {
      pcm,
      pcm_len,
      sample_format: result.sample_format,
      sample_rate: result.sample_rate,
    }
  }
}
impl Drop for WhisprsSpeechResult {
  fn drop(&mut self) {
    unsafe {
      let pcm = std::slice::from_raw_parts_mut(self.pcm, self.pcm_len);
      let _pcm = Box::from_raw(std::ptr::from_mut::<[c_uchar]>(pcm));
    }
  }
}
#[repr(C)]
pub struct WhisprsSpeechSynthesizerMetadata {
  pub name: *mut c_char,
  pub supports_speaking_to_audio_data: bool,
  pub supports_speech_parameters: bool,
}
impl From<SpeechSynthesizerMetadata> for WhisprsSpeechSynthesizerMetadata {
  fn from(synthesizer: SpeechSynthesizerMetadata) -> Self {
    WhisprsSpeechSynthesizerMetadata {
      name: CString::new(synthesizer.name).unwrap().into_raw(),
      supports_speaking_to_audio_data: synthesizer.supports_speaking_to_audio_data,
      supports_speech_parameters: synthesizer.supports_speech_parameters,
    }
  }
}
impl Drop for WhisprsSpeechSynthesizerMetadata {
  fn drop(&mut self) {
    let _name = unsafe { CString::from_raw(self.name) };
  }
}
#[repr(C)]
pub struct WhisprsBrailleBackendMetadata {
  pub name: *mut c_char,
  pub priority: c_uchar,
}
impl From<BrailleBackendMetadata> for WhisprsBrailleBackendMetadata {
  fn from(backend: BrailleBackendMetadata) -> Self {
    WhisprsBrailleBackendMetadata {
      name: CString::new(backend.name).unwrap().into_raw(),
      priority: backend.priority,
    }
  }
}
impl Drop for WhisprsBrailleBackendMetadata {
  fn drop(&mut self) {
    let _name = unsafe { CString::from_raw(self.name) };
  }
}
#[repr(C)]
pub struct WhisprsVoice {
  pub synthesizer: *mut WhisprsSpeechSynthesizerMetadata,
  pub display_name: *mut c_char,
  pub name: *mut c_char,
  pub languages: *mut *mut c_char,
  pub languages_len: usize,
  pub priority: c_uchar,
}
impl From<Voice> for WhisprsVoice {
  fn from(voice: Voice) -> Self {
    let languages: Vec<*mut c_char> = voice
      .languages
      .into_iter()
      .map(|language| CString::new(language).unwrap().into_raw())
      .collect();
    let languages_len = languages.len();
    let mut languages_box = languages.into_boxed_slice();
    let languages = languages_box.as_mut_ptr();
    std::mem::forget(languages_box);
    WhisprsVoice {
      synthesizer: Box::into_raw(Box::new(voice.synthesizer.into())),
      display_name: CString::new(voice.display_name).unwrap().into_raw(),
      name: CString::new(voice.name).unwrap().into_raw(),
      languages,
      languages_len,
      priority: voice.priority,
    }
  }
}
impl Drop for WhisprsVoice {
  fn drop(&mut self) {
    unsafe {
      let _synthesizer = Box::from_raw(self.synthesizer);
      let _display_name = CString::from_raw(self.display_name);
      let _name = CString::from_raw(self.name);
      let languages = std::slice::from_raw_parts_mut(self.languages, self.languages_len);
      let languages = Box::from_raw(std::ptr::from_mut::<[*mut c_char]>(languages));
      let _languages: Vec<CString> = languages
        .iter()
        .map(|language| CString::from_raw(*language))
        .collect();
    }
  }
}
