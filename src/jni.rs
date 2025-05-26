use crate::error::OutputError;
use crate::*;
use ::jni::objects::*;
use ::jni::sys::*;
use ::jni::JNIEnv;
fn error_to_exception_class(error: &OutputError) -> String {
  "org/whisprs/error/".to_owned()
    + match error {
      OutputError::BackendNotFound(_) => "BackendNotFoundException",
      OutputError::AudioDataNotSupported(_) => "AudioDataNotSupportedException",
      OutputError::SpeechNotSupported(_) => "SpeechNotSupportedException",
      OutputError::BrailleNotSupported(_) => "BrailleNotSupportedException",
      OutputError::VoiceNotFound(_) => "VoiceNotFoundException",
      OutputError::LanguageNotFound(_) => "LanguageNotFoundException",
      OutputError::NoVoices => "NoVoicesException",
      OutputError::NoBrailleBackends => "NoBrailleBackendsException",
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
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_initialize<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) {
  match initialize() {
    Ok(()) => (),
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      return;
    }
  }
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listVoices<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let voices = match list_voices() {
    Ok(voices) => voices,
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      return Default::default();
    }
  };
  let voice_class = env
    .find_class("org/mcaccess/whisprs/metadata/Voice")
    .unwrap();
  let speech_synthesizer_metadata_class = env
    .find_class("org/mcaccess/whisprs/metadata/SpeechSynthesizerMetadata")
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
          "(Lorg/mcaccess/whisprs/metadata/SpeechSynthesizerMetadata;Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;B)V",
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
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listBrailleBackends<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let backends = match list_braille_backends() {
    Ok(backends) => backends,
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      return Default::default();
    }
  };
  let braille_backend_class = env
    .find_class("org/mcaccess/whisprs/metadata/BrailleBackend")
    .unwrap();
  let backends = backends
    .into_iter()
    .map(|backend| {
      let name = env.new_string(&backend.name).unwrap();
      let priority = backend.priority as i8;
      env
        .new_object(
          &braille_backend_class,
          "(Ljava/lang/String;B)V",
          &[JValue::Object(&name), JValue::Byte(priority)],
        )
        .unwrap()
    })
    .collect::<Vec<JObject>>();
  let array = env
    .new_object_array(
      backends.len().try_into().unwrap(),
      &braille_backend_class,
      JObject::null(),
    )
    .unwrap();
  for (index, backend) in backends.into_iter().enumerate() {
    env
      .set_object_array_element(&array, index.try_into().unwrap(), backend)
      .unwrap();
  }
  array
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
  let result = match speak_to_audio_data(
    synthesizer.as_deref(),
    voice.as_deref(),
    language.as_deref(),
    rate,
    volume,
    pitch,
    &text,
  ) {
    Ok(result) => result,
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      return Default::default();
    }
  };
  let buffer = env.byte_array_from_slice(&result.pcm).unwrap();
  let speech_result_class = env
    .find_class("org/mcaccess/whisprs/audio/SpeechResult")
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
  let interrupt: bool = match interrupt {
    JNI_FALSE => false,
    _ => true,
  };
  match speak_to_audio_output(
    synthesizer.as_deref(),
    voice.as_deref(),
    language.as_deref(),
    rate,
    volume,
    pitch,
    &text,
    interrupt,
  ) {
    Ok(()) => (),
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      return;
    }
  }
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_stopSpeech<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  synthesizer: JString<'local>,
) {
  let null = JObject::null();
  let synthesizer: Option<String> = if env.is_same_object(&synthesizer, &null).unwrap() {
    None
  } else {
    Some(env.get_string(&synthesizer).unwrap().into())
  };
  match stop_speech(synthesizer.as_deref()) {
    Ok(()) => (),
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      return;
    }
  }
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_braille<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  backend: JString<'local>,
  text: JString<'local>,
) {
  let null = JObject::null();
  let backend: Option<String> = if env.is_same_object(&backend, &null).unwrap() {
    None
  } else {
    Some(env.get_string(&backend).unwrap().into())
  };
  let text: String = env.get_string(&text).unwrap().into();
  match braille(backend.as_deref(), &text) {
    Ok(()) => (),
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      return;
    }
  }
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
  let braille_backend: Option<String> = if env.is_same_object(&braille_backend, &null).unwrap() {
    None
  } else {
    Some(env.get_string(&braille_backend).unwrap().into())
  };
  let text: String = env.get_string(&text).unwrap().into();
  let interrupt: bool = match interrupt {
    JNI_FALSE => false,
    _ => true,
  };
  match output(
    synthesizer.as_deref(),
    voice.as_deref(),
    language.as_deref(),
    rate,
    volume,
    pitch,
    braille_backend.as_deref(),
    &text,
    interrupt,
  ) {
    Ok(()) => (),
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      return;
    }
  }
}
