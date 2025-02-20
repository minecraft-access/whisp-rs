#![allow(non_upper_case_globals)] use espeakng_sys::*;
use std::os::raw::{c_char,c_short,c_int};
use std::ffi::{c_void,CStr,CString};
use lazy_static::lazy_static;
use std::cell::Cell;
use std::sync::Mutex;
use std::iter::once;
use crate::speech_synthesizer::{SpeechError,SpeechResult,SpeechSynthesizer,Voice};
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
    let output: espeak_AUDIO_OUTPUT = espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_SYNCHRONOUS;
    let path_cstr = CString::new(".")?;
    let result = EspeakNg { sample_rate: unsafe { espeak_Initialize(output, 0, path_cstr.as_ptr(), 0).try_into()? }};
    Ok(result)
  }
  fn name(&self) -> String {
    "eSpeak NG".to_owned()
  }
  fn min_rate(&self) -> u32 {
    espeakRATE_MINIMUM
  }
  fn max_rate(&self) -> u32 {
    espeakRATE_MAXIMUM
  }
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError> {
    let mut voice_spec = espeak_VOICE { name: std::ptr::null(), languages: std::ptr::null(), identifier: std::ptr::null(), gender: 0, age: 0, variant: 0, xx1: 0, score: 0, spare: std::ptr::null_mut() };
    let voices_ptr = unsafe { espeak_ListVoices(&mut voice_spec) };
    let mut voices_ptr_copy = voices_ptr.clone();
    let mut count: usize = 0;
    while unsafe { !(*voices_ptr_copy).is_null() } {
      count+=1;
      voices_ptr_copy = unsafe { voices_ptr_copy.add(1) };
    }
    let voices_slice = unsafe { std::slice::from_raw_parts(voices_ptr, count) };
    let voices = unsafe { voices_slice.into_iter().map(|voice| {
      let name = CStr::from_ptr((**voice).name).to_str().unwrap().to_owned();
      let identifier = CStr::from_ptr((**voice).identifier).to_str().unwrap().to_owned();
      let mut languages_ptr_copy = (**voice).languages.clone();
      let mut string_start = languages_ptr_copy.clone();
      let mut priority = 0;
      let mut last_byte_was_null = true;
      let mut last_byte_was_priority = false;
      let mut languages: Vec<(u8, String)> = Vec::new();
      while !(last_byte_was_null && (*languages_ptr_copy)==0) {
        match (last_byte_was_null, last_byte_was_priority, *languages_ptr_copy) {
          (true, _, byte) => {
            priority = byte;
            last_byte_was_null = false;
            last_byte_was_priority = true
          },
          (_, true, byte) => {
            string_start = languages_ptr_copy.clone();
            last_byte_was_priority = false;
            if byte==0 {
              last_byte_was_null = true;
              languages.push((priority.try_into().unwrap(), CStr::from_ptr(string_start).to_str().unwrap().to_owned()));
            };
          },
          (_, _, 0) => {
            last_byte_was_null = true;
            languages.push((priority.try_into().unwrap(), CStr::from_ptr(string_start).to_str().unwrap().to_owned()));
          },
          (_, _, _) => {}
        };
        languages_ptr_copy = languages_ptr_copy.add(1);
      };
      let language = languages.into_iter().min_by_key(|tuple| tuple.0);
      (name, identifier, language.unwrap_or((0, "empty".to_owned())).1)
    })
    .collect::<Vec<(String, String, String)>>() };
    let variants = voices.iter().filter(|voice| voice.2=="variant");
    let main_voices = voices.iter().filter(|voice| voice.2!="variant");
    let voices = main_voices.flat_map(|voice|
      once(Voice { synthesizer: self.name(), display_name: voice.0.clone(), name: voice.0.clone(), language: voice.2.clone() })
        .chain(variants.clone().map(move |variant| Voice { synthesizer: self.name(), display_name: voice.0.clone()+" ("+&variant.0+")", name: voice.0.clone()+"+"+&variant.1.replace("!v/", ""), language: voice.2.clone() })));
    Ok(voices.collect::<Vec<Voice>>())
  }
  fn speak(&self, voice: &str, rate: u32, volume: u8, pitch: u8, text: &str) -> Result<SpeechResult, SpeechError> {
    let voice_cstr = CString::new(voice)?;
    handle_espeak_error(unsafe { espeak_SetVoiceByName(voice_cstr.as_ptr()) })?;
    if rate < self.min_rate() || rate > self.max_rate() { return Err(SpeechError { message: "Rate is out of range".to_owned() }) };
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakRATE, rate.try_into()?, 0) })?;
    if volume > 100 { return Err(SpeechError { message: "Volume is out of range".to_owned() }) };
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakVOLUME, (volume*2).try_into()?, 0) })?;
    if pitch > 100 { return Err(SpeechError { message: "Pitch is out of range".to_owned() }) };
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakPITCH, pitch.try_into()?, 0) })?;
    unsafe { espeak_SetSynthCallback(Some(synth_callback)) };
    let text_cstr = CString::new(text)?;
    let position = 0u32;
    let position_type: espeak_POSITION_TYPE = 0;
    let end_position = 0u32;
    let flags = espeakCHARS_AUTO;
    let identifier = std::ptr::null_mut();
    let user_data = std::ptr::null_mut();
    handle_espeak_error(unsafe { espeak_Synth(text_cstr.as_ptr() as *const c_void, text_cstr.count_bytes(), position, position_type, end_position, flags, identifier, user_data) })?;
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
