use jni::JNIEnv;
use jni::objects::{JByteArray,JClass,JObject,JValue,JObjectArray,JString};
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
  let voice_class = env.find_class("dev/emassey0135/audionavigation/speech/Voice").expect("Failed to get class: dev.emassey0135.audionavigation.speech.Voice");
  let voices = voices.into_iter().map(|voice| {
    let synthesizer = env.new_string(&voice.synthesizer.name()).expect("Failed to create Java string");
    let display_name = env.new_string(&voice.display_name).expect("Failed to create Java string");
    let name = env.new_string(&voice.name).expect("Failed to create Java string");
    let language = env.new_string(&voice.language).expect("Failed to create Java string");
    env.new_object(&voice_class, "(Ljava.lang.String;Ljava.lang.String;Ljava.lang.String;Ljava.lang.String;)V", &[JValue::Object(&synthesizer), JValue::Object(&display_name), JValue::Object(&name), JValue::Object(&language)]).expect("Failed to create Voice object")
  })
  .collect::<Vec<JObject>>();
  let array = env.new_object_array(voices.len().try_into().unwrap(), voice_class, JObject::null()).expect("Failed to create Java array");
  let mut index: usize = 0;
  for voice in voices {
    env.set_object_array_element(&array, index.try_into().unwrap(), voice).expect("Failed to add Voice to array");
    index+=1
  }
  array
}
