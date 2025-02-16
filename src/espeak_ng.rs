#![allow(non_upper_case_globals)] use espeakng_sys::*;
use std::os::raw::{c_char,c_short,c_int};
use std::ffi::{c_void,CStr,CString};
use lazy_static::lazy_static;
use std::cell::Cell;
use std::sync::Mutex;
use crate::speech_synthesizer::{SpeechError,SpeechResult,SpeechSynthesizer};
lazy_static! {
  static ref BUFFER: Mutex<Cell<Vec<u8>>> = Mutex::new(Cell::new(Vec::default()));
}
fn handle_espeak_error(error: espeak_ERROR) -> Result<(), SpeechError> {
  match error {
    espeak_ERROR_EE_OK => Ok(()),
    error => Err(SpeechError { message: format!("eSpeak-NG error code: {}", error) })
  }
}
#[derive(Debug)] pub struct EspeakNg {
  sample_rate: u32
}
impl SpeechSynthesizer for EspeakNg {
  fn new() -> Result<Self, SpeechError> {
    let output: espeak_AUDIO_OUTPUT = espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_RETRIEVAL;
    let path_cstr = CString::new(".")?;
    let result = EspeakNg { sample_rate: unsafe { espeak_Initialize(output, 0, path_cstr.as_ptr(), 0).try_into()? }};
    Ok(result)
  }
  fn min_rate(&self) -> u32 {
    espeakRATE_MINIMUM
  }
  fn max_rate(&self) -> u32 {
    espeakRATE_MAXIMUM
  }
  fn list_voices(&self, language: &str) -> Result<Vec<String>, SpeechError> {
    let language_cstr = CString::new(language)?;
    let mut voice_spec = espeak_VOICE { name: std::ptr::null(), languages: language_cstr.as_ptr(), identifier: std::ptr::null(), gender: 0, age: 0, variant: 0, xx1: 0, score: 0, spare: std::ptr::null_mut() };
    let voices = unsafe { espeak_ListVoices(&mut voice_spec) };
    let mut voices_copy = voices.clone();
    let mut count: usize = 0;
    while unsafe { !(*voices_copy).is_null() } {
      count+=1;
      voices_copy = unsafe { voices_copy.add(1) };
    }
    let voices_slice = unsafe { std::slice::from_raw_parts(voices, count) };
    let voice_names = unsafe { voices_slice.into_iter().map(|voice| CStr::from_ptr((**voice).name).to_str().unwrap().to_owned()).collect::<Vec<String>>() };
    Ok(voice_names)
  }
  fn speak(&self, voice: &str, rate: u32, volume: u8, pitch: u8, pitch_range: u8, text: &str) -> Result<SpeechResult, SpeechError> {
    let voice_cstr = CString::new(voice)?;
    handle_espeak_error(unsafe { espeak_SetVoiceByName(voice_cstr.as_ptr()) })?;
    if rate < self.min_rate() || rate > self.max_rate() { return Err(SpeechError { message: "Rate is out of range".to_owned() }) };
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakRATE, rate.try_into()?, 0) })?;
    if volume > 100 { return Err(SpeechError { message: "Volume is out of range".to_owned() }) };
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakVOLUME, volume.try_into()?, 0) })?;
    if pitch > 100 { return Err(SpeechError { message: "Pitch is out of range".to_owned() }) };
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakPITCH, pitch.try_into()?, 0) })?;
    if pitch_range > 100 { return Err(SpeechError { message: "Pitch_range is out of range".to_owned() }) };
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakRANGE, pitch_range.try_into()?, 0) })?;
    unsafe { espeak_SetSynthCallback(Some(synth_callback)) };
    let text_cstr = CString::new(text)?;
    let position = 0u32;
    let position_type: espeak_POSITION_TYPE = 0;
    let end_position = 0u32;
    let flags = espeakCHARS_AUTO;
    let identifier = std::ptr::null_mut();
    let user_data = std::ptr::null_mut();
    handle_espeak_error(unsafe { espeak_Synth(text_cstr.as_ptr() as *const c_void, text_cstr.count_bytes(), position, position_type, end_position, flags, identifier, user_data) })?;
    handle_espeak_error(unsafe { espeak_Synchronize() })?;
    let result = BUFFER.lock().unwrap().take();
    Ok(SpeechResult { pcm: result, sample_rate: self.sample_rate })
  }
}
unsafe extern "C" fn synth_callback(wav: *mut c_short, sample_count: c_int, _events: *mut espeak_EVENT) -> c_int {
  if !wav.is_null() {
    let wav_slice = std::slice::from_raw_parts_mut(wav as *mut c_char, 2*sample_count as usize);
    let mut wav_vec = wav_slice.into_iter().map(|byte| byte.clone() as u8).collect::<Vec<u8>>();
    let mut buffer = BUFFER.lock().unwrap().take();
    buffer.append(&mut wav_vec);
    BUFFER.lock().unwrap().set(buffer);
  }
  0
}
