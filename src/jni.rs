use crate::*;
use ::jni::objects::*;
use ::jni::sys::*;
use ::jni::JNIEnv;
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_initialize<'local>(
  _env: JNIEnv<'local>,
  _class: JClass<'local>,
) {
  initialize().unwrap()
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listVoices<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let voices = list_voices().unwrap();
  let voice_class = env.find_class("org/mcaccess/whisprs/Voice").unwrap();
  let speech_synthesizer_metadata_class = env
    .find_class("org/mcaccess/whisprs/SpeechSynthesizerMetadata")
    .unwrap();
  let string_class = env.find_class("java/lang/String").unwrap();
  let voices = voices
    .into_iter()
    .map(|voice| {
      let synthesizer_name = env.new_string(&voice.synthesizer.name).unwrap();
      let synthesizer_supports_speaking_to_audio_data = if voice.synthesizer.supports_speaking_to_audio_data { JNI_TRUE } else { JNI_FALSE };
      let synthesizer_supports_speech_parameters = if voice.synthesizer.supports_speech_parameters { JNI_TRUE } else { JNI_FALSE };
      let synthesizer = env
        .new_object(
          &speech_synthesizer_metadata_class,
          "(Ljava/lang/String;ZZ)V",
          &[
            JValue::Object(&synthesizer_name),
            JValue::Bool(synthesizer_supports_speaking_to_audio_data),
            JValue::Bool(synthesizer_supports_speech_parameters),
          ],
        )
        .unwrap();
      let display_name = env.new_string(&voice.display_name).unwrap();
      let name = env.new_string(&voice.name).unwrap();
      let languages = env
        .new_object_array(
          voice.languages.len().try_into().unwrap(),
          &string_class,
          JObject::null(),
        )
        .unwrap();
      for (index, language) in voice.languages.iter().enumerate() {
        let language = env
          .new_string(language)
          .unwrap();
        env
          .set_object_array_element(&languages, index.try_into().unwrap(), language)
          .unwrap();
      }
      let priority = voice.priority as i8;
      env
        .new_object(
          &voice_class,
          "(Lorg/mcaccess/whisprs/SpeechSynthesizerMetadata;Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;B)V",
          &[
            JValue::Object(&synthesizer),
            JValue::Object(&display_name),
            JValue::Object(&name),
            JValue::Object(&languages),
            JValue::Byte(priority),
          ],
        )
        .unwrap()
    })
    .collect::<Vec<JObject>>();
  let array = env
    .new_object_array(
      voices.len().try_into().unwrap(),
      &voice_class,
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
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_speak<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  synthesizer: JString<'local>,
  voice: JString<'local>,
  language: JString<'local>,
  rate: JObject<'local>,
  volume: JObject<'local>,
  pitch: JObject<'local>,
  text: JString<'local>,
) -> JObject<'local> {
  let null = JObject::null();
  let synthesizer: Option<String> = if env.is_same_object(&synthesizer, &null).unwrap() {
    None
  } else {
    Some(env.get_string(&synthesizer).unwrap().into())
  };
  let voice: Option<String> = if env.is_same_object(&voice, &null).unwrap() {
    None
  } else {
    Some(env.get_string(&voice).unwrap().into())
  };
  let language: Option<String> = if env.is_same_object(&language, &null).unwrap() {
    None
  } else {
    Some(env.get_string(&language).unwrap().into())
  };
  let rate: Option<u8> = if env.is_same_object(&rate, &null).unwrap() {
    None
  } else {
    Some(
      env
        .call_method(&rate, "byteValue", "()B", &[])
        .unwrap()
        .b()
        .unwrap() as u8,
    )
  };
  let volume: Option<u8> = if env.is_same_object(&volume, &null).unwrap() {
    None
  } else {
    Some(
      env
        .call_method(&volume, "byteValue", "()B", &[])
        .unwrap()
        .b()
        .unwrap() as u8,
    )
  };
  let pitch: Option<u8> = if env.is_same_object(&pitch, &null).unwrap() {
    None
  } else {
    Some(
      env
        .call_method(&pitch, "byteValue", "()B", &[])
        .unwrap()
        .b()
        .unwrap() as u8,
    )
  };
  let text: String = env.get_string(&text).unwrap().into();
  let result = speak_to_audio_data(
    synthesizer.as_deref(),
    voice.as_deref(),
    language.as_deref(),
    rate,
    volume,
    pitch,
    &text,
  )
  .unwrap();
  let buffer = env.byte_array_from_slice(&result.pcm).unwrap();
  let speech_result_class = env.find_class("org/mcaccess/whisprs/SpeechResult").unwrap();
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
