#![allow(non_upper_case_globals)] use espeakng_sys::*;
use std::os::raw::{c_char,c_short,c_int};
use std::ffi::{c_void,CString};
use lazy_static::lazy_static;
use std::cell::Cell;
use std::sync::atomic::{AtomicI32,Ordering};
use std::sync::Mutex;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::objects::JByteBuffer;
pub struct SpeechResult {
  pub pcm: Vec<i16>,
  pub sample_rate: i32
}
lazy_static! {
  static ref SAMPLE_RATE: AtomicI32 = AtomicI32::new(22050);
  static ref BUFFER: Mutex<Cell<Vec<i16>>> = Mutex::new(Cell::new(Vec::default()));
}
pub fn initialize() {
  let output: espeak_AUDIO_OUTPUT = espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_RETRIEVAL;
  let path: *const c_char = std::ptr::null();
  SAMPLE_RATE.store(unsafe { espeak_Initialize(output, 0, path, 0) }, Ordering::Release);
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
    espeak_Synth(text_cstr.as_ptr() as *const c_void, text.len(), position, position_type, end_position, flags, identifier, user_data);
  }
  match unsafe { espeak_Synchronize() } {
    espeak_ERROR_EE_OK => {},
    espeak_ERROR_EE_INTERNAL_ERROR => { todo!() },
    _ => unreachable!()
  }
  let result = BUFFER.lock().unwrap().take();
  SpeechResult { pcm: result, sample_rate: SAMPLE_RATE.load(Ordering::Acquire) }
}
unsafe extern "C" fn synth_callback(wav: *mut c_short, sample_count: c_int, _events: *mut espeak_EVENT) -> c_int {
  if !wav.is_null() {
    let wav_slice = std::slice::from_raw_parts_mut(wav, sample_count as usize);
    let mut wav_vec = wav_slice.into_iter().map(|sample| sample.clone() as i16).collect::<Vec<i16>>();
    let mut buffer = BUFFER.lock().unwrap().take();
    buffer.append(&mut wav_vec);
    BUFFER.lock().unwrap().set(buffer);
  }
  0
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_Speech_initialize<'local>(_env: JNIEnv<'local>, _class: JClass<'local>) {
  initialize();
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_Speech_speak<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>, text: JString<'local>) -> JByteBuffer<'local> {
  let text: String = env.get_string(&text).expect("Failed to get Java string").into();
  let mut result: SpeechResult = speak(&text);
  let buffer = unsafe { env.new_direct_byte_buffer(result.pcm.as_mut_ptr() as *mut u8, result.pcm.len()).expect("Failed to create byte buffer") };
  buffer
}
