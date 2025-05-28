use crate::audio::{SampleFormat, SpeechResult};
use crate::metadata::{BrailleBackendMetadata, SpeechSynthesizerMetadata, Voice};
use crate::{
  braille, initialize, list_braille_backends, list_speech_synthesizers,
  list_speech_synthesizers_supporting_audio_data, list_voices, output, speak_to_audio_data,
  speak_to_audio_output, stop_speech,
};
use std::ffi::{c_char, c_uchar, c_uint, CStr, CString};
pub type WhisprsSampleFormat = SampleFormat;
#[repr(C)]
pub struct WhisprsSpeechResult {
  pub pcm: *mut u8,
  pub pcm_len: usize,
  pub sample_format: WhisprsSampleFormat,
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
unsafe fn optional_c_string_to_rust(string: &*const c_char) -> Option<&str> {
  if string.is_null() {
    None
  } else {
    Some(CStr::from_ptr(*string).to_str().unwrap())
  }
}
unsafe fn optional_c_byte_to_rust(byte: *const c_uchar) -> Option<u8> {
  if byte.is_null() {
    None
  } else {
    Some(*byte)
  }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_initialize() {
  initialize().unwrap();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_list_voices(
  synthesizer: *const c_char,
  name: *const c_char,
  language: *const c_char,
  needs_audio_data: bool,
  voices_ptr: *mut *mut *mut WhisprsVoice,
  voices_len: *mut usize,
) {
  let synthesizer = optional_c_string_to_rust(&synthesizer);
  let name = optional_c_string_to_rust(&name);
  let language = optional_c_string_to_rust(&language);
  let voices: Vec<*mut WhisprsVoice> = list_voices(synthesizer, name, language, needs_audio_data)
    .unwrap()
    .into_iter()
    .map(|voice| Box::into_raw(Box::new(voice.into())))
    .collect();
  *voices_len = voices.len();
  let mut voices = voices.into_boxed_slice();
  *voices_ptr = voices.as_mut_ptr();
  std::mem::forget(voices);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_free_voice_list(
  voices: *mut *mut WhisprsVoice,
  voices_len: usize,
) {
  let voices = std::slice::from_raw_parts_mut(voices, voices_len);
  let voices = Box::from_raw(std::ptr::from_mut::<[*mut WhisprsVoice]>(voices));
  let _voices: Vec<Box<WhisprsVoice>> = voices.iter().map(|ptr| Box::from_raw(*ptr)).collect();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_list_speech_synthesizers(
  synthesizers_ptr: *mut *mut *mut WhisprsSpeechSynthesizerMetadata,
  synthesizers_len: *mut usize,
) {
  let synthesizers: Vec<*mut WhisprsSpeechSynthesizerMetadata> = list_speech_synthesizers()
    .unwrap()
    .into_iter()
    .map(|synthesizer| Box::into_raw(Box::new(synthesizer.into())))
    .collect();
  *synthesizers_len = synthesizers.len();
  let mut synthesizers = synthesizers.into_boxed_slice();
  *synthesizers_ptr = synthesizers.as_mut_ptr();
  std::mem::forget(synthesizers);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_list_speech_synthesizers_supporting_audio_data(
  synthesizers_ptr: *mut *mut *mut WhisprsSpeechSynthesizerMetadata,
  synthesizers_len: *mut usize,
) {
  let synthesizers: Vec<*mut WhisprsSpeechSynthesizerMetadata> =
    list_speech_synthesizers_supporting_audio_data()
      .unwrap()
      .into_iter()
      .map(|synthesizer| Box::into_raw(Box::new(synthesizer.into())))
      .collect();
  *synthesizers_len = synthesizers.len();
  let mut synthesizers = synthesizers.into_boxed_slice();
  *synthesizers_ptr = synthesizers.as_mut_ptr();
  std::mem::forget(synthesizers);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_free_speech_synthesizer_list(
  synthesizers: *mut *mut WhisprsSpeechSynthesizerMetadata,
  synthesizers_len: usize,
) {
  let synthesizers = std::slice::from_raw_parts_mut(synthesizers, synthesizers_len);
  let synthesizers =
    Box::from_raw(std::ptr::from_mut::<[*mut WhisprsSpeechSynthesizerMetadata]>(synthesizers));
  let _synthesizers: Vec<Box<WhisprsSpeechSynthesizerMetadata>> =
    synthesizers.iter().map(|ptr| Box::from_raw(*ptr)).collect();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_list_braille_backends(
  backends_ptr: *mut *mut *mut WhisprsBrailleBackendMetadata,
  backends_len: *mut usize,
) {
  let backends: Vec<*mut WhisprsBrailleBackendMetadata> = list_braille_backends()
    .unwrap()
    .into_iter()
    .map(|backend| Box::into_raw(Box::new(backend.into())))
    .collect();
  *backends_len = backends.len();
  let mut backends = backends.into_boxed_slice();
  *backends_ptr = backends.as_mut_ptr();
  std::mem::forget(backends);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_free_braille_backend_list(
  backends: *mut *mut WhisprsBrailleBackendMetadata,
  backends_len: usize,
) {
  let backends = std::slice::from_raw_parts_mut(backends, backends_len);
  let backends = Box::from_raw(std::ptr::from_mut::<[*mut WhisprsBrailleBackendMetadata]>(
    backends,
  ));
  let _backends: Vec<Box<WhisprsBrailleBackendMetadata>> =
    backends.iter().map(|ptr| Box::from_raw(*ptr)).collect();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_speak_to_audio_data(
  synthesizer: *const c_char,
  voice: *const c_char,
  language: *const c_char,
  rate: *const c_uchar,
  volume: *const c_uchar,
  pitch: *const c_uchar,
  text: *const c_char,
  result_ptr: *mut *mut WhisprsSpeechResult,
) {
  let synthesizer = optional_c_string_to_rust(&synthesizer);
  let voice = optional_c_string_to_rust(&voice);
  let language = optional_c_string_to_rust(&language);
  let rate = optional_c_byte_to_rust(rate);
  let volume = optional_c_byte_to_rust(volume);
  let pitch = optional_c_byte_to_rust(pitch);
  let text = CStr::from_ptr(text).to_str().unwrap();
  let result =
    speak_to_audio_data(synthesizer, voice, language, rate, volume, pitch, text).unwrap();
  *result_ptr = Box::into_raw(Box::new(result.into()));
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_free_speech_result(result: *mut WhisprsSpeechResult) {
  let _result = Box::from_raw(result);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_speak_to_audio_output(
  synthesizer: *const c_char,
  voice: *const c_char,
  language: *const c_char,
  rate: *const c_uchar,
  volume: *const c_uchar,
  pitch: *const c_uchar,
  text: *const c_char,
  interrupt: bool,
) {
  let synthesizer = optional_c_string_to_rust(&synthesizer);
  let voice = optional_c_string_to_rust(&voice);
  let language = optional_c_string_to_rust(&language);
  let rate = optional_c_byte_to_rust(rate);
  let volume = optional_c_byte_to_rust(volume);
  let pitch = optional_c_byte_to_rust(pitch);
  let text = CStr::from_ptr(text).to_str().unwrap();
  speak_to_audio_output(
    synthesizer,
    voice,
    language,
    rate,
    volume,
    pitch,
    text,
    interrupt,
  )
  .unwrap();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_stop_speech(synthesizer: *const c_char) {
  let synthesizer = optional_c_string_to_rust(&synthesizer);
  stop_speech(synthesizer).unwrap();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_braille(backend: *const c_char, text: *const c_char) {
  let backend = optional_c_string_to_rust(&backend);
  let text = CStr::from_ptr(text).to_str().unwrap();
  braille(backend, text).unwrap();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whisprs_output(
  synthesizer: *const c_char,
  voice: *const c_char,
  language: *const c_char,
  rate: *const c_uchar,
  volume: *const c_uchar,
  pitch: *const c_uchar,
  braille_backend: *const c_char,
  text: *const c_char,
  interrupt: bool,
) {
  let synthesizer = optional_c_string_to_rust(&synthesizer);
  let voice = optional_c_string_to_rust(&voice);
  let language = optional_c_string_to_rust(&language);
  let rate = optional_c_byte_to_rust(rate);
  let volume = optional_c_byte_to_rust(volume);
  let pitch = optional_c_byte_to_rust(pitch);
  let braille_backend = optional_c_string_to_rust(&braille_backend);
  let text = CStr::from_ptr(text).to_str().unwrap();
  output(
    synthesizer,
    voice,
    language,
    rate,
    volume,
    pitch,
    braille_backend,
    text,
    interrupt,
  )
  .unwrap();
}
