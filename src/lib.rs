#![allow(non_upper_case_globals)] use espeakng_sys::*;
use std::os::raw::{c_char,c_short,c_int};
use std::ffi::{c_void,CStr,CString};
use lazy_static::lazy_static;
use std::cell::Cell;
use std::sync::atomic::{AtomicI32,Ordering};
use std::sync::Mutex;
use jni::JNIEnv;
use jni::objects::{JByteArray,JClass,JObjectArray,JString};
use jni::sys::jint;
pub struct SpeechResult {
  pub pcm: Vec<u8>,
  pub sample_rate: i32
}
lazy_static! {
  static ref SAMPLE_RATE: AtomicI32 = AtomicI32::new(22050);
  static ref BUFFER: Mutex<Cell<Vec<u8>>> = Mutex::new(Cell::new(Vec::default()));
}
pub fn handle_espeak_error(error: espeak_ERROR, message: &str) {
  match error {
    espeak_ERROR_EE_OK => {},
    _ => panic!("{}", message)
  }
}
pub fn initialize(path: &str) {
  let output: espeak_AUDIO_OUTPUT = espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_RETRIEVAL;
  let path_cstr = CString::new(path).expect("Failed to convert text to CString");
  SAMPLE_RATE.store(unsafe { espeak_Initialize(output, 0, path_cstr.as_ptr(), 0) }, Ordering::Release);
}
pub fn speak(text: &str) -> SpeechResult {
  unsafe {
    espeak_SetSynthCallback(Some(synth_callback));
  };
  let text_cstr = CString::new(text).expect("Failed to convert text to CString");
  let position = 0u32;
  let position_type: espeak_POSITION_TYPE = 0;
  let end_position = 0u32;
  let flags = espeakCHARS_AUTO;
  let identifier = std::ptr::null_mut();
  let user_data = std::ptr::null_mut();
  unsafe {
    espeak_Synth(text_cstr.as_ptr() as *const c_void, text_cstr.count_bytes(), position, position_type, end_position, flags, identifier, user_data);
  }
  handle_espeak_error(unsafe { espeak_Synchronize() }, "eSpeak internal error");
  let result = BUFFER.lock().unwrap().take();
  SpeechResult { pcm: result, sample_rate: SAMPLE_RATE.load(Ordering::Acquire) }
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
pub fn set_rate(rate: i32) {
  handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakRATE, rate, 0) }, "Error setting eSpeak speech rate");
}
pub fn set_volume(volume: i32) {
  handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakVOLUME, volume, 0) }, "Error setting eSpeak speech volume");
}
pub fn set_pitch(pitch: i32) {
  handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakPITCH, pitch, 0) }, "Error setting eSpeak speech pitch");
}
pub fn set_pitch_range(pitch_range: i32) {
  handle_espeak_error(unsafe { espeak_SetParameter(espeak_PARAMETER_espeakRANGE, pitch_range, 0) }, "Error setting eSpeak speech pitch range");
}
pub fn list_voices(language: &str) -> Vec<String> {
  let language_cstr = CString::new(language).expect("Failed to convert text to CString");
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
  voice_names
}
pub fn set_voice(name: &str) {
  let name_cstr = CString::new(name).expect("Failed to convert text to CString");
  handle_espeak_error(unsafe { espeak_SetVoiceByName(name_cstr.as_ptr()) }, "Error setting eSpeak speech pitch range");
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_initialize<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>, path: JString<'local>) {
  let path: String = env.get_string(&path).expect("Failed to get Java string").into();
  initialize(&path);
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_speak<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>, text: JString<'local>) -> JByteArray<'local> {
  let text: String = env.get_string(&text).expect("Failed to get Java string").into();
  let result: SpeechResult = speak(&text);
  let pcm = result.pcm;
  let buffer = env.byte_array_from_slice(&pcm).expect("Failed to create byte buffer");
  buffer
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_setRate<'local>(_env: JNIEnv<'local>, _class: JClass<'local>, rate: jint) {
  set_rate(rate)
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_setVolume<'local>(_env: JNIEnv<'local>, _class: JClass<'local>, volume: jint) {
  set_volume(volume)
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_setPitch<'local>(_env: JNIEnv<'local>, _class: JClass<'local>, pitch: jint) {
  set_pitch(pitch)
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_setPitchRange<'local>(_env: JNIEnv<'local>, _class: JClass<'local>, pitch_range: jint) {
  set_pitch_range(pitch_range)
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_listVoices<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>, language: JString<'local>) -> JObjectArray<'local> {
  let language: String = env.get_string(&language).expect("Failed to get Java string").into();
  let names = list_voices(&language);
  let string_class = env.find_class("java/lang/String").expect("Failed to get class: java.lang.string");
  let empty_string = env.new_string("").expect("Failed to create empty string");
  let array = env.new_object_array(names.len().try_into().unwrap(), string_class, empty_string).expect("Failed to create Java array");
  let mut index: usize = 0;
  while index<names.len() {
    let name_jstring = env.new_string(&names[index]).expect("Failed to create Java string");
    env.set_object_array_element(&array, index.try_into().unwrap(), name_jstring).expect("Failed to add string to array");
    index+=1
  }
  array
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_setVoice<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>, name: JString<'local>) {
  let name: String = env.get_string(&name).expect("Failed to get Java string").into();
  set_voice(&name)
}
