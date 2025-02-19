use jni::JNIEnv;
use jni::objects::{JClass,JObject,JValue,JObjectArray,JString};
use jni::sys::{jbyte,jint};
use std::sync::OnceLock;
use crate::speech_synthesizer::{SpeechResult,SpeechSynthesizer};
use crate::espeak_ng::EspeakNg;
use crate::sapi::Sapi;
mod speech_synthesizer;
mod espeak_ng;
mod sapi;
  static SAPI: OnceLock<Sapi> = OnceLock::new();
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_Native_initialize<'local>(_env: JNIEnv<'local>, _class: JClass<'local>) {
  SAPI.set(Sapi::new().unwrap()).unwrap()
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_Native_speak<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>, voice: JString<'local>, rate: jint, volume: jbyte, pitch: jbyte, text: JString<'local>) -> JObject<'local> {
  let voice: String = env.get_string(&voice).unwrap().into();
  let text: String = env.get_string(&text).unwrap().into();
  let result: SpeechResult = SAPI.get().unwrap().speak(&voice, rate.try_into().unwrap(), volume.try_into().unwrap(), pitch.try_into().unwrap(), &text).unwrap();
  let buffer = env.byte_array_from_slice(&result.pcm).unwrap();
  let speech_result_class = env.find_class("dev/emassey0135/audionavigation/speech/SpeechResult").unwrap();
  env.new_object(&speech_result_class, "([BI)V", &[JValue::Object(&buffer), JValue::Int(result.sample_rate.try_into().unwrap())]).unwrap()
}
#[no_mangle] pub extern "system" fn Java_dev_emassey0135_audionavigation_speech_Native_listVoices<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>) -> JObjectArray<'local> {
  let voices = SAPI.get().unwrap().list_voices().unwrap();
  let voice_class = env.find_class("dev/emassey0135/audionavigation/speech/Voice").unwrap();
  let voices = voices.into_iter().map(|voice| {
    let synthesizer = env.new_string(&voice.synthesizer.name()).unwrap();
    let display_name = env.new_string(&voice.display_name).unwrap();
    let name = env.new_string(&voice.name).unwrap();
    let language = env.new_string(&voice.language).unwrap();
    env.new_object(&voice_class, "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V", &[JValue::Object(&synthesizer), JValue::Object(&display_name), JValue::Object(&name), JValue::Object(&language)]).unwrap()
  })
  .collect::<Vec<JObject>>();
  let array = env.new_object_array(voices.len().try_into().unwrap(), voice_class, JObject::null()).unwrap();
  let mut index: usize = 0;
  for voice in voices {
    env.set_object_array_element(&array, index.try_into().unwrap(), voice).unwrap();
    index+=1
  }
  array
}
