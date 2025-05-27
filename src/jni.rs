use crate::error::OutputError;
use crate::{
  braille, initialize, list_braille_backends, list_speech_synthesizers,
  list_speech_synthesizers_supporting_audio_data, list_voices, output, speak_to_audio_data,
  speak_to_audio_output, stop_speech, SpeechSynthesizerMetadata,
};
use ::jni::errors::Error;
use ::jni::objects::{JClass, JObject, JObjectArray, JString, JValue};
use ::jni::sys::{jboolean, JNI_FALSE, JNI_TRUE};
use ::jni::JNIEnv;
fn error_to_exception_class(error: &OutputError) -> String {
  "org/mcaccess/whisprs/error/".to_owned()
    + match error {
      OutputError::BackendNotFound(_) => "BackendNotFoundException",
      OutputError::AudioDataNotSupported(_) => "AudioDataNotSupportedException",
      OutputError::SpeechNotSupported(_) => "SpeechNotSupportedException",
      OutputError::BrailleNotSupported(_) => "BrailleNotSupportedException",
      OutputError::VoiceNotFound(_) => "VoiceNotFoundException",
      OutputError::LanguageNotFound(_) => "LanguageNotFoundException",
      OutputError::NoVoices => "NoVoicesException",
      OutputError::NoBrailleBackends => "NoBrailleBackendsException",
      OutputError::NoBackends => "NoBackendsException",
      OutputError::InvalidRate(_) => "InvalidRateException",
      OutputError::InvalidVolume(_) => "InvalidVolumeException",
      OutputError::InvalidPitch(_) => "InvalidPitchException",
      OutputError::SpeakFailed {
        backend: _,
        voice: _,
        error: _,
      } => "SpeakFailedException",
      OutputError::StopSpeechFailed {
        backend: _,
        error: _,
      } => "StopSpeechFailedException",
      OutputError::BrailleFailed {
        backend: _,
        error: _,
      } => "BrailleFailedException",
      OutputError::InitializeFailed(_) => "InitializeFailedException",
      OutputError::Unknown(_) => "UnknownException",
    }
}
fn throw_exception_when_needed<T: std::default::Default>(
  env: &mut JNIEnv,
  result: Result<T, OutputError>,
) -> T {
  match result {
    Ok(value) => value,
    Err(OutputError::Unknown(error)) => {
      if let Some(Error::JavaException) = error.downcast_ref::<Error>() {
        Default::default()
      } else {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
        Default::default()
      }
    }
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      Default::default()
    }
  }
}
fn jni_optional_string_to_rust(
  env: &mut JNIEnv,
  string: &JString,
) -> Result<Option<String>, OutputError> {
  let null = JObject::null();
  let string = if env
    .is_same_object(string, &null)
    .map_err(OutputError::into_unknown)?
  {
    None
  } else {
    Some(
      env
        .get_string(string)
        .map_err(OutputError::into_unknown)?
        .into(),
    )
  };
  Ok(string)
}
#[allow(clippy::cast_sign_loss)]
fn jni_optional_byte_to_rust(env: &mut JNIEnv, byte: &JObject) -> Result<Option<u8>, OutputError> {
  let null = JObject::null();
  let byte = if env
    .is_same_object(byte, &null)
    .map_err(OutputError::into_unknown)?
  {
    None
  } else {
    Some(
      env
        .call_method(byte, "byteValue", "()B", &[])
        .map_err(OutputError::into_unknown)?
        .b()
        .map_err(OutputError::into_unknown)? as u8,
    )
  };
  Ok(byte)
}
fn speech_synthesizer_metadata_to_jni<'local>(
  env: &mut JNIEnv<'local>,
  synthesizer: &SpeechSynthesizerMetadata,
) -> Result<JObject<'local>, OutputError> {
  let speech_synthesizer_metadata_class = env
    .find_class("org/mcaccess/whisprs/metadata/SpeechSynthesizerMetadata")
    .map_err(OutputError::into_unknown)?;
  let name = env
    .new_string(&synthesizer.name)
    .map_err(OutputError::into_unknown)?;
  let supports_speaking_to_audio_data = if synthesizer.supports_speaking_to_audio_data {
    JNI_TRUE
  } else {
    JNI_FALSE
  };
  let supports_speech_parameters = if synthesizer.supports_speech_parameters {
    JNI_TRUE
  } else {
    JNI_FALSE
  };
  let synthesizer = env
    .new_object(
      &speech_synthesizer_metadata_class,
      "(Ljava/lang/String;ZZ)V",
      &[
        JValue::Object(&name),
        JValue::Bool(supports_speaking_to_audio_data),
        JValue::Bool(supports_speech_parameters),
      ],
    )
    .map_err(OutputError::into_unknown)?;
  Ok(synthesizer)
}
fn objects_to_jni_array<'local>(
  env: &mut JNIEnv<'local>,
  class: &JClass<'local>,
  objects: &[JObject<'local>],
) -> Result<JObjectArray<'local>, OutputError> {
  let array = env
    .new_object_array(
      objects
        .len()
        .try_into()
        .map_err(OutputError::into_unknown)?,
      class,
      JObject::null(),
    )
    .map_err(OutputError::into_unknown)?;
  for (index, object) in objects.iter().enumerate() {
    env
      .set_object_array_element(
        &array,
        index.try_into().map_err(OutputError::into_unknown)?,
        object,
      )
      .map_err(OutputError::into_unknown)?;
  }
  Ok(array)
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_initialize<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) {
  let closure = || initialize();
  throw_exception_when_needed(&mut env, closure());
}
#[allow(clippy::cast_possible_wrap)]
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listVoices<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  synthesizer: JString<'local>,
  name: JString<'local>,
  language: JString<'local>,
  needs_audio_data: jboolean,
) -> JObjectArray<'local> {
  let mut closure = || {
    let synthesizer = jni_optional_string_to_rust(&mut env, &synthesizer)?;
    let name = jni_optional_string_to_rust(&mut env, &name)?;
    let language = jni_optional_string_to_rust(&mut env, &language)?;
    let needs_audio_data: bool = needs_audio_data != JNI_FALSE;
    let voices = list_voices(
      synthesizer.as_deref(),
      name.as_deref(),
      language.as_deref(),
      needs_audio_data,
    )?;
    let voice_class = env
      .find_class("org/mcaccess/whisprs/metadata/Voice")
      .map_err(OutputError::into_unknown)?;
    let string_class = env
      .find_class("java/lang/String")
      .map_err(OutputError::into_unknown)?;
    let voices = voices
      .into_iter()
      .map(|voice| {
      let synthesizer = speech_synthesizer_metadata_to_jni(&mut env, &voice.synthesizer)?;
      let display_name = env.new_string(&voice.display_name)?;
      let name = env.new_string(&voice.name)?;
      let languages = voice.languages.into_iter().map(|language| env.new_string(language).map_err(OutputError::into_unknown).map(std::convert::Into::into)).collect::<Result<Vec<JObject>, OutputError>>()?;
      let languages = objects_to_jni_array(&mut env, &string_class, &languages)?;
      let priority = voice.priority as i8;
      let voice = env
        .new_object(
          &voice_class,
          "(Lorg/mcaccess/whisprs/metadata/SpeechSynthesizerMetadata;Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;B)V",
          &[
            JValue::Object(&synthesizer),
            JValue::Object(&display_name),
            JValue::Object(&name),
            JValue::Object(&languages),
            JValue::Byte(priority),
          ],
        )?;
        Ok::<_, anyhow::Error>(voice)
    })
    .collect::<Result<Vec<JObject>, anyhow::Error>>().map_err(OutputError::into_unknown)?;
    let voices = objects_to_jni_array(&mut env, &voice_class, &voices)?;
    Ok(voices)
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result)
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listSpeechSynthesizers<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let mut closure = || {
    let speech_synthesizer_metadata_class = env
      .find_class("org/mcaccess/whisprs/metadata/SpeechSynthesizerMetadata")
      .map_err(OutputError::into_unknown)?;
    let synthesizers = list_speech_synthesizers()?;
    let synthesizers = synthesizers
      .iter()
      .map(|synthesizer| speech_synthesizer_metadata_to_jni(&mut env, synthesizer))
      .collect::<Result<Vec<JObject>, OutputError>>()?;
    let synthesizers =
      objects_to_jni_array(&mut env, &speech_synthesizer_metadata_class, &synthesizers)?;
    Ok(synthesizers)
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result)
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listSpeechSynthesizersSupportingAudioData<
  'local,
>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let mut closure = || {
    let synthesizers = list_speech_synthesizers_supporting_audio_data()?;
    let speech_synthesizer_metadata_class = env
      .find_class("org/mcaccess/whisprs/metadata/SpeechSynthesizerMetadata")
      .map_err(OutputError::into_unknown)?;
    let synthesizers = synthesizers
      .iter()
      .map(|synthesizer| speech_synthesizer_metadata_to_jni(&mut env, synthesizer))
      .collect::<Result<Vec<JObject>, OutputError>>()?;
    let synthesizers =
      objects_to_jni_array(&mut env, &speech_synthesizer_metadata_class, &synthesizers)?;
    Ok(synthesizers)
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result)
}
#[allow(clippy::cast_possible_wrap)]
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listBrailleBackends<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let mut closure = || {
    let backends = list_braille_backends()?;
    let braille_backend_metadata_class = env
      .find_class("org/mcaccess/whisprs/metadata/BrailleBackendMetadata")
      .map_err(OutputError::into_unknown)?;
    let backends = backends
      .into_iter()
      .map(|backend| {
        let name = env.new_string(&backend.name)?;
        let priority = backend.priority as i8;
        let backend = env.new_object(
          &braille_backend_metadata_class,
          "(Ljava/lang/String;B)V",
          &[JValue::Object(&name), JValue::Byte(priority)],
        )?;
        Ok::<_, anyhow::Error>(backend)
      })
      .collect::<Result<Vec<JObject>, anyhow::Error>>()
      .map_err(OutputError::into_unknown)?;
    let backends = objects_to_jni_array(&mut env, &braille_backend_metadata_class, &backends)?;
    Ok(backends)
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result)
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_speakToAudioData<'local>(
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
  let mut closure = || {
    let synthesizer = jni_optional_string_to_rust(&mut env, &synthesizer)?;
    let voice = jni_optional_string_to_rust(&mut env, &voice)?;
    let language = jni_optional_string_to_rust(&mut env, &language)?;
    let rate = jni_optional_byte_to_rust(&mut env, &rate)?;
    let volume = jni_optional_byte_to_rust(&mut env, &volume)?;
    let pitch = jni_optional_byte_to_rust(&mut env, &pitch)?;
    let text: String = env
      .get_string(&text)
      .map_err(OutputError::into_unknown)?
      .into();
    let result = speak_to_audio_data(
      synthesizer.as_deref(),
      voice.as_deref(),
      language.as_deref(),
      rate,
      volume,
      pitch,
      &text,
    )?;
    let buffer = env
      .byte_array_from_slice(&result.pcm)
      .map_err(OutputError::into_unknown)?;
    let speech_result_class = env
      .find_class("org/mcaccess/whisprs/audio/SpeechResult")
      .map_err(OutputError::into_unknown)?;
    let result = env
      .new_object(
        &speech_result_class,
        "([BBI)V",
        &[
          JValue::Object(&buffer),
          JValue::Byte(result.sample_format as i8),
          JValue::Int(
            result
              .sample_rate
              .try_into()
              .map_err(OutputError::into_unknown)?,
          ),
        ],
      )
      .map_err(OutputError::into_unknown)?;
    Ok(result)
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result)
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_speakToAudioOutput<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  synthesizer: JString<'local>,
  voice: JString<'local>,
  language: JString<'local>,
  rate: JObject<'local>,
  volume: JObject<'local>,
  pitch: JObject<'local>,
  text: JString<'local>,
  interrupt: jboolean,
) {
  let mut closure = || {
    let synthesizer = jni_optional_string_to_rust(&mut env, &synthesizer)?;
    let voice = jni_optional_string_to_rust(&mut env, &voice)?;
    let language = jni_optional_string_to_rust(&mut env, &language)?;
    let rate = jni_optional_byte_to_rust(&mut env, &rate)?;
    let volume = jni_optional_byte_to_rust(&mut env, &volume)?;
    let pitch = jni_optional_byte_to_rust(&mut env, &pitch)?;
    let text: String = env
      .get_string(&text)
      .map_err(OutputError::into_unknown)?
      .into();
    let interrupt: bool = interrupt != JNI_FALSE;
    speak_to_audio_output(
      synthesizer.as_deref(),
      voice.as_deref(),
      language.as_deref(),
      rate,
      volume,
      pitch,
      &text,
      interrupt,
    )
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result);
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_stopSpeech<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  synthesizer: JString<'local>,
) {
  let mut closure = || {
    let synthesizer = jni_optional_string_to_rust(&mut env, &synthesizer)?;
    stop_speech(synthesizer.as_deref())
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result);
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_braille<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  backend: JString<'local>,
  text: JString<'local>,
) {
  let mut closure = || {
    let backend = jni_optional_string_to_rust(&mut env, &backend)?;
    let text: String = env
      .get_string(&text)
      .map_err(OutputError::into_unknown)?
      .into();
    braille(backend.as_deref(), &text)
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result);
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_output<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  synthesizer: JString<'local>,
  voice: JString<'local>,
  language: JString<'local>,
  rate: JObject<'local>,
  volume: JObject<'local>,
  pitch: JObject<'local>,
  braille_backend: JString<'local>,
  text: JString<'local>,
  interrupt: jboolean,
) {
  let mut closure = || {
    let synthesizer = jni_optional_string_to_rust(&mut env, &synthesizer)?;
    let voice = jni_optional_string_to_rust(&mut env, &voice)?;
    let language = jni_optional_string_to_rust(&mut env, &language)?;
    let rate = jni_optional_byte_to_rust(&mut env, &rate)?;
    let volume = jni_optional_byte_to_rust(&mut env, &volume)?;
    let pitch = jni_optional_byte_to_rust(&mut env, &pitch)?;
    let braille_backend = jni_optional_string_to_rust(&mut env, &braille_backend)?;
    let text: String = env
      .get_string(&text)
      .map_err(OutputError::into_unknown)?
      .into();
    let interrupt: bool = interrupt != JNI_FALSE;
    output(
      synthesizer.as_deref(),
      voice.as_deref(),
      language.as_deref(),
      rate,
      volume,
      pitch,
      braille_backend.as_deref(),
      &text,
      interrupt,
    )
  };
  let result = closure();
  throw_exception_when_needed(&mut env, result);
}
