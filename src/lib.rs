use jni::JNIEnv;
use jni::objects::{JByteArray,JClass,JObject,JObjectArray,JString};
use jni::sys::{jbyte,jint};
use std::sync::OnceLock;
use crate::speech_synthesizer::{SpeechResult,SpeechSynthesizer};
use crate::espeak_ng::EspeakNg;
mod speech_synthesizer;
mod espeak_ng;
  static ESPEAK_NG: OnceLock<EspeakNg> = OnceLock::new();
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_initialize<'local>(_env: JNIEnv<'local>, _class: JClass<'local>) {
  ESPEAK_NG.set(EspeakNg::new().unwrap()).unwrap()
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_speak<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>, voice: JString<'local>, rate: jint, volume: jbyte, pitch: jbyte, pitch_range: jbyte, text: JString<'local>) -> JByteArray<'local> {
  let voice: String = env.get_string(&voice).expect("Failed to get Java string").into();
  let text: String = env.get_string(&text).expect("Failed to get Java string").into();
  let result: SpeechResult = ESPEAK_NG.get().unwrap().speak(&voice, rate.try_into().unwrap(), volume.try_into().unwrap(), pitch.try_into().unwrap(), pitch_range.try_into().unwrap(), &text).unwrap();
  let pcm = result.pcm;
  let buffer = env.byte_array_from_slice(&pcm).expect("Failed to create byte buffer");
  buffer
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_EspeakNative_listVoices<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>) -> JObjectArray<'local> {
  let voices = ESPEAK_NG.get().unwrap().list_voices().unwrap();
  let string_class = env.find_class("java/lang/String").expect("Failed to get class: java.lang.String");
  let string_array_class = env.find_class("[Ljava/lang/String;").expect("Failed to get class: [java.lang.String]");
  let empty_string = env.new_string("").expect("Failed to create empty string");
  let arrays = voices.into_iter().map(|voice| {
    let array = env.new_object_array(2, &string_class, &empty_string).expect("Failed to create Java array");
    let name_jstring = env.new_string(&voice.name).expect("Failed to create Java string");
    env.set_object_array_element(&array, 0, name_jstring).expect("Failed to add string to array");
    let language_jstring = env.new_string(&voice.language).expect("Failed to create Java string");
    env.set_object_array_element(&array, 1, language_jstring).expect("Failed to add string to array");
    array
  })
  .collect::<Vec<JObjectArray>>();
  let array = env.new_object_array(arrays.len().try_into().unwrap(), string_array_class, JObject::null()).expect("Failed to create Java array");
  let mut index: usize = 0;
  for voice_array in arrays {
    env.set_object_array_element(&array, index.try_into().unwrap(), voice_array).expect("Failed to add array to array");
    index+=1
  }
  array
}
