#![allow(non_upper_case_globals)]
use crate::audio::*;
use crate::backends::*;
use crate::error::OutputError;
use crate::metadata::Voice;
use anyhow::anyhow;
use espeakng_sys::*;
use lazy_static::lazy_static;
use std::cell::Cell;
use std::ffi::{c_void, CStr, CString};
use std::iter::once;
use std::os::raw::{c_char, c_int, c_short};
use std::sync::Mutex;
lazy_static! {
  static ref BUFFER: Mutex<Cell<Vec<u8>>> = Mutex::new(Cell::new(Vec::new()));
}
fn handle_espeak_error(error: espeak_ERROR) -> Result<(), anyhow::Error> {
  match error {
    espeak_ERROR_EE_OK => Ok(()),
    error => Err(anyhow!("eSpeak NG error: {}", error)),
  }
}
pub struct EspeakNg {
  default_voice: String,
  sample_rate: u32,
}
impl Backend for EspeakNg {
  fn new() -> Result<Self, OutputError> {
    let output: espeak_AUDIO_OUTPUT = espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_SYNCHRONOUS;
    let path_cstr = CString::new(".").map_err(OutputError::into_unknown)?;
    let sample_rate: u32 = unsafe {
      espeak_Initialize(output, 0, path_cstr.as_ptr(), 0)
        .try_into()
        .map_err(OutputError::into_unknown)?
    };
    let default_voice = "en".to_owned();
    let result = EspeakNg {
      default_voice,
      sample_rate,
    };
    Ok(result)
  }
  fn name(&self) -> String {
    "eSpeak NG".to_owned()
  }
  fn list_voices(&self) -> Result<Vec<Voice>, OutputError> {
    let mut voice_spec = espeak_VOICE {
      name: std::ptr::null(),
      languages: std::ptr::null(),
      identifier: std::ptr::null(),
      gender: 0,
      age: 0,
      variant: 0,
      xx1: 0,
      score: 0,
      spare: std::ptr::null_mut(),
    };
    let voices_ptr = unsafe { espeak_ListVoices(&mut voice_spec) };
    let mut voices_ptr_copy = voices_ptr;
    let mut count: usize = 0;
    while unsafe { !(*voices_ptr_copy).is_null() } {
      count += 1;
      voices_ptr_copy = unsafe { voices_ptr_copy.add(1) };
    }
    let voices_slice = unsafe { std::slice::from_raw_parts(voices_ptr, count) };
    let voices = unsafe {
      voices_slice
        .iter()
        .flat_map(|voice| {
          let name = CStr::from_ptr((**voice).name).to_str()?.to_owned();
          let identifier = CStr::from_ptr((**voice).identifier).to_str()?.to_owned();
          let mut languages_ptr_copy = (**voice).languages;
          let mut string_start = languages_ptr_copy;
          let mut priority = 0;
          let mut last_byte_was_null = true;
          let mut last_byte_was_priority = false;
          let mut languages: Vec<(u8, String)> = Vec::new();
          while !(last_byte_was_null && (*languages_ptr_copy) == 0) {
            match (
              last_byte_was_null,
              last_byte_was_priority,
              *languages_ptr_copy,
            ) {
              (true, _, byte) => {
                priority = byte;
                last_byte_was_null = false;
                last_byte_was_priority = true
              }
              (_, true, byte) => {
                string_start = languages_ptr_copy;
                last_byte_was_priority = false;
                if byte == 0 {
                  last_byte_was_null = true;
                  languages.push((
                    priority.try_into()?,
                    CStr::from_ptr(string_start).to_str()?.to_owned(),
                  ));
                };
              }
              (_, _, 0) => {
                last_byte_was_null = true;
                languages.push((
                  priority.try_into()?,
                  CStr::from_ptr(string_start).to_str()?.to_owned(),
                ));
              }
              (_, _, _) => {}
            };
            languages_ptr_copy = languages_ptr_copy.add(1);
          }
          let language = languages.into_iter().min_by_key(|tuple| tuple.0);
          let language = match language {
            None => vec![],
            Some(language) => vec![language.1],
          };
          Ok::<(String, String, Vec<String>), anyhow::Error>((name, identifier, language))
        })
        .collect::<Vec<(String, String, Vec<String>)>>()
    };
    let variants = voices
      .iter()
      .filter(|voice| voice.2.first().is_some_and(|value| value == "variant"));
    let main_voices = voices
      .iter()
      .filter(|voice| voice.2.first().is_some_and(|value| value != "variant"));
    let voices = main_voices.flat_map(|voice| {
      once(Voice {
        synthesizer: self.speech_metadata().unwrap(),
        display_name: voice.0.clone(),
        name: voice.0.clone(),
        languages: voice.2.clone(),
        priority: 3,
      })
      .chain(variants.clone().map(move |variant| Voice {
        synthesizer: self.speech_metadata().unwrap(),
        display_name: voice.0.clone() + " (" + &variant.0 + ")",
        name: voice.0.clone() + "+" + &variant.1.replace("!v/", ""),
        languages: voice.2.clone(),
        priority: 3,
      }))
    });
    Ok(voices.collect::<Vec<Voice>>())
  }
  fn as_speech_synthesizer_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    Some(self)
  }
  fn as_speech_synthesizer_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    None
  }
  fn as_braille_backend(&self) -> Option<&dyn BrailleBackend> {
    None
  }
}
impl SpeechSynthesizerToAudioData for EspeakNg {
  fn supports_speech_parameters(&self) -> bool {
    true
  }
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> Result<SpeechResult, OutputError> {
    match (voice, language) {
      (None, None) => {
        let voice_cstr = CString::new(&*self.default_voice).map_err(OutputError::into_unknown)?;
        handle_espeak_error(unsafe { espeak_SetVoiceByName(voice_cstr.as_ptr()) })
          .map_err(|_| OutputError::into_voice_not_found(&self.default_voice))?;
      }
      (Some(voice), _) => {
        let voice_cstr = CString::new(voice).map_err(OutputError::into_unknown)?;
        handle_espeak_error(unsafe { espeak_SetVoiceByName(voice_cstr.as_ptr()) })
          .map_err(|_| OutputError::into_voice_not_found(voice))?;
      }
      (_, Some(language)) => {
        let language_cstr = CString::new(language).map_err(OutputError::into_unknown)?;
        let mut voice_spec = espeak_VOICE {
          name: std::ptr::null(),
          languages: language_cstr.as_ptr(),
          identifier: std::ptr::null(),
          gender: 0,
          age: 0,
          variant: 0,
          xx1: 0,
          score: 0,
          spare: std::ptr::null_mut(),
        };
        handle_espeak_error(unsafe { espeak_SetVoiceByProperties(&mut voice_spec) })
          .map_err(|_| OutputError::into_language_not_found(language))?;
      }
    };
    let rate = rate.unwrap_or(50) as f64;
    let rate = (rate / 100.0) * ((espeakRATE_MAXIMUM - espeakRATE_MINIMUM) as f64)
      + (espeakRATE_MINIMUM as f64);
    let rate = (rate.round()) as i32;
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakRATE, rate, 0) })
      .map_err(|err| {
        OutputError::into_speak_failed(
          &self.name(),
          voice.unwrap_or(language.unwrap_or(&self.default_voice)),
          err,
        )
      })?;
    let volume = volume.unwrap_or(100) as i32;
    handle_espeak_error(unsafe {
      espeak_SetParameter(espeak_PARAMETER_espeakVOLUME, volume * 2, 0)
    })
    .map_err(|err| {
      OutputError::into_speak_failed(
        &self.name(),
        voice.unwrap_or(language.unwrap_or(&self.default_voice)),
        err,
      )
    })?;
    let pitch = pitch.unwrap_or(50) as i32;
    handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakPITCH, pitch, 0) })
      .map_err(|err| {
        OutputError::into_speak_failed(
          &self.name(),
          voice.unwrap_or(language.unwrap_or(&self.default_voice)),
          err,
        )
      })?;
    unsafe { espeak_SetSynthCallback(Some(synth_callback)) };
    let text_cstr = CString::new(text).map_err(OutputError::into_unknown)?;
    let position = 0u32;
    let position_type: espeak_POSITION_TYPE = 0;
    let end_position = 0u32;
    let flags = espeakCHARS_AUTO;
    let identifier = std::ptr::null_mut();
    let user_data = std::ptr::null_mut();
    handle_espeak_error(unsafe {
      espeak_Synth(
        text_cstr.as_ptr() as *const c_void,
        text_cstr.count_bytes(),
        position,
        position_type,
        end_position,
        flags,
        identifier,
        user_data,
      )
    })
    .map_err(|err| {
      OutputError::into_speak_failed(
        &self.name(),
        voice.unwrap_or(language.unwrap_or(&self.default_voice)),
        err,
      )
    })?;
    let result = BUFFER
      .lock()
      .map_err(|_| OutputError::into_unknown(anyhow!("Failed to lock the eSpeak audio buffer")))?
      .take();
    Ok(SpeechResult {
      pcm: result,
      sample_format: SampleFormat::S16,
      sample_rate: self.sample_rate,
    })
  }
}
unsafe extern "C" fn synth_callback(
  wav: *mut c_short,
  sample_count: c_int,
  _events: *mut espeak_EVENT,
) -> c_int {
  if !wav.is_null() {
    let wav_slice = std::slice::from_raw_parts_mut(wav as *mut c_char, 2 * sample_count as usize);
    let mut wav_vec = wav_slice
      .iter()
      .map(|byte| *byte as u8)
      .collect::<Vec<u8>>();
    let mut buffer = BUFFER.lock().unwrap().take();
    buffer.append(&mut wav_vec);
    BUFFER.lock().unwrap().set(buffer);
  }
  0
}
