#![deny(clippy::all)]
//#![deny(clippy::pedantic)]
use crate::speech::*;
use jni::objects::{JClass, JObject, JObjectArray, JString, JValue};
use jni::sys::jbyte;
use jni::JNIEnv;
mod backends;
pub mod speech;
mod speech_synthesizer;
#[no_mangle]
pub extern "system" fn Java_dev_emassey0135_audionavigation_client_speech_Native_initialize<
  'local,
>(
  _env: JNIEnv<'local>,
  _class: JClass<'local>,
) {
  initialize().unwrap()
}
#[no_mangle]
pub extern "system" fn Java_dev_emassey0135_audionavigation_client_speech_Native_listVoices<
  'local,
>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let voices = list_voices().unwrap();
  let voice_class = env
    .find_class("dev/emassey0135/audionavigation/client/speech/Voice")
    .unwrap();
  let voices = voices
    .into_iter()
    .map(|voice| {
      let synthesizer = env.new_string(&voice.synthesizer.name).unwrap();
      let display_name = env.new_string(&voice.display_name).unwrap();
      let name = env.new_string(&voice.name).unwrap();
      let language = env
        .new_string(
          voice
            .languages
            .first()
            .map_or("none".to_owned(), |string| string.to_owned()),
        )
        .unwrap();
      env
        .new_object(
          &voice_class,
          "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
          &[
            JValue::Object(&synthesizer),
            JValue::Object(&display_name),
            JValue::Object(&name),
            JValue::Object(&language),
          ],
        )
        .unwrap()
    })
    .collect::<Vec<JObject>>();
  let array = env
    .new_object_array(
      voices.len().try_into().unwrap(),
      voice_class,
      JObject::null(),
    )
    .unwrap();
  for (index, voice) in voices.into_iter().enumerate() {
    env
      .set_object_array_element(&array, index.try_into().unwrap(), voice)
      .unwrap();
  }
  array
}
#[no_mangle]
pub extern "system" fn Java_dev_emassey0135_audionavigation_client_speech_Native_speak<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  synthesizer: JString<'local>,
  voice: JString<'local>,
  language: JString<'local>,
  rate: jbyte,
  volume: jbyte,
  pitch: jbyte,
  text: JString<'local>,
) -> JObject<'local> {
  let synthesizer: String = env.get_string(&synthesizer).unwrap().into();
  let voice: String = env.get_string(&voice).unwrap().into();
  let language: String = env.get_string(&language).unwrap().into();
  let text: String = env.get_string(&text).unwrap().into();
  let rate = Some(rate.try_into().unwrap());
  let volume = Some(volume.try_into().unwrap());
  let pitch = Some(pitch.try_into().unwrap());
  let result = speak_to_audio_data(
    Some(&synthesizer),
    Some(&voice),
    Some(&language),
    rate,
    volume,
    pitch,
    &text,
  )
  .unwrap();
  let buffer = env.byte_array_from_slice(&result.pcm).unwrap();
  let speech_result_class = env
    .find_class("dev/emassey0135/audionavigation/client/speech/SpeechResult")
    .unwrap();
  env
    .new_object(
      &speech_result_class,
      "([BBI)V",
      &[
        JValue::Object(&buffer),
        JValue::Byte(result.sample_format as i8),
        JValue::Int(result.sample_rate.try_into().unwrap()),
      ],
    )
    .unwrap()
}
