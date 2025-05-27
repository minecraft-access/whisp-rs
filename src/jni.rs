use crate::error::OutputError;
use crate::*;
use ::jni::errors::Error;
use ::jni::objects::*;
use ::jni::sys::*;
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
  let closure = || initialize();
  match closure() {
    Ok(()) => (),
    Err(OutputError::Unknown(error)) => match error.downcast_ref::<Error>() {
      Some(Error::JavaException) => (),
      _ => {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      }
    },
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
    }
  }
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listVoices<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let mut closure = || {
    let voices = list_voices()?;
    let voice_class = env
      .find_class("org/mcaccess/whisprs/metadata/Voice")
      .map_err(OutputError::into_unknown)?;
    let speech_synthesizer_metadata_class = env
      .find_class("org/mcaccess/whisprs/metadata/SpeechSynthesizerMetadata")
      .map_err(OutputError::into_unknown)?;
    let string_class = env
      .find_class("java/lang/String")
      .map_err(OutputError::into_unknown)?;
    let voices = voices
    .into_iter()
    .map(|voice| {
      let synthesizer_name = env.new_string(&voice.synthesizer.name)?;
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
        ?;
      let display_name = env.new_string(&voice.display_name)?;
      let name = env.new_string(&voice.name)?;
      let languages = env
        .new_object_array(
          voice.languages.len().try_into()?,
          &string_class,
          JObject::null(),
        )
        ?;
      for (index, language) in voice.languages.iter().enumerate() {
        let language = env
          .new_string(language)
          ?;
        env
          .set_object_array_element(&languages, index.try_into()?, language)
          ?;
      }
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
    let array = env
      .new_object_array(
        voices.len().try_into().map_err(OutputError::into_unknown)?,
        &voice_class,
        JObject::null(),
      )
      .map_err(OutputError::into_unknown)?;
    for (index, voice) in voices.into_iter().enumerate() {
      env
        .set_object_array_element(
          &array,
          index.try_into().map_err(OutputError::into_unknown)?,
          voice,
        )
        .map_err(OutputError::into_unknown)?;
    }
    Ok(array)
  };
  match closure() {
    Ok(voices) => voices,
    Err(OutputError::Unknown(error)) => match error.downcast_ref::<Error>() {
      Some(Error::JavaException) => Default::default(),
      _ => {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
        Default::default()
      }
    },
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      Default::default()
    }
  }
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_listBrailleBackends<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
) -> JObjectArray<'local> {
  let mut closure = || {
    let backends = list_braille_backends()?;
    let braille_backend_class = env
      .find_class("org/mcaccess/whisprs/metadata/BrailleBackend")
      .map_err(OutputError::into_unknown)?;
    let backends = backends
      .into_iter()
      .map(|backend| {
        let name = env.new_string(&backend.name)?;
        let priority = backend.priority as i8;
        let backend = env.new_object(
          &braille_backend_class,
          "(Ljava/lang/String;B)V",
          &[JValue::Object(&name), JValue::Byte(priority)],
        )?;
        Ok::<_, anyhow::Error>(backend)
      })
      .collect::<Result<Vec<JObject>, anyhow::Error>>()
      .map_err(OutputError::into_unknown)?;
    let array = env
      .new_object_array(
        backends
          .len()
          .try_into()
          .map_err(OutputError::into_unknown)?,
        &braille_backend_class,
        JObject::null(),
      )
      .map_err(OutputError::into_unknown)?;
    for (index, backend) in backends.into_iter().enumerate() {
      env
        .set_object_array_element(
          &array,
          index.try_into().map_err(OutputError::into_unknown)?,
          backend,
        )
        .map_err(OutputError::into_unknown)?;
    }
    Ok(array)
  };
  match closure() {
    Ok(backends) => backends,
    Err(OutputError::Unknown(error)) => match error.downcast_ref::<Error>() {
      Some(Error::JavaException) => Default::default(),
      _ => {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
        Default::default()
      }
    },
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      Default::default()
    }
  }
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
    let null = JObject::null();
    let synthesizer: Option<String> = if env
      .is_same_object(&synthesizer, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&synthesizer)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let voice: Option<String> = if env
      .is_same_object(&voice, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&voice)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let language: Option<String> = if env
      .is_same_object(&language, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&language)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let rate: Option<u8> = if env
      .is_same_object(&rate, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&rate, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
    let volume: Option<u8> = if env
      .is_same_object(&volume, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&volume, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
    let pitch: Option<u8> = if env
      .is_same_object(&pitch, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&pitch, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
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
  match closure() {
    Ok(result) => result,
    Err(OutputError::Unknown(error)) => match error.downcast_ref::<Error>() {
      Some(Error::JavaException) => Default::default(),
      _ => {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
        Default::default()
      }
    },
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      Default::default()
    }
  }
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
    let null = JObject::null();
    let synthesizer: Option<String> = if env
      .is_same_object(&synthesizer, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&synthesizer)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let voice: Option<String> = if env
      .is_same_object(&voice, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&voice)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let language: Option<String> = if env
      .is_same_object(&language, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&language)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let rate: Option<u8> = if env
      .is_same_object(&rate, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&rate, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
    let volume: Option<u8> = if env
      .is_same_object(&volume, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&volume, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
    let pitch: Option<u8> = if env
      .is_same_object(&pitch, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&pitch, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
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
  match closure() {
    Ok(()) => (),
    Err(OutputError::Unknown(error)) => match error.downcast_ref::<Error>() {
      Some(Error::JavaException) => (),
      _ => {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      }
    },
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
    }
  }
}
#[no_mangle]
pub extern "system" fn Java_org_mcaccess_whisprs_Whisprs_stopSpeech<'local>(
  mut env: JNIEnv<'local>,
  _class: JClass<'local>,
  synthesizer: JString<'local>,
) {
  let mut closure = || {
    let null = JObject::null();
    let synthesizer: Option<String> = if env
      .is_same_object(&synthesizer, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&synthesizer)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    stop_speech(synthesizer.as_deref())
  };
  match closure() {
    Ok(()) => (),
    Err(OutputError::Unknown(error)) => match error.downcast_ref::<Error>() {
      Some(Error::JavaException) => (),
      _ => {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      }
    },
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
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
  let mut closure = || {
    let null = JObject::null();
    let backend: Option<String> = if env
      .is_same_object(&backend, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&backend)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let text: String = env
      .get_string(&text)
      .map_err(OutputError::into_unknown)?
      .into();
    braille(backend.as_deref(), &text)
  };
  match closure() {
    Ok(()) => (),
    Err(OutputError::Unknown(error)) => match error.downcast_ref::<Error>() {
      Some(Error::JavaException) => (),
      _ => {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      }
    },
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
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
  let mut closure = || {
    let null = JObject::null();
    let synthesizer: Option<String> = if env
      .is_same_object(&synthesizer, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&synthesizer)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let voice: Option<String> = if env
      .is_same_object(&voice, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&voice)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let language: Option<String> = if env
      .is_same_object(&language, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&language)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
    let rate: Option<u8> = if env
      .is_same_object(&rate, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&rate, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
    let volume: Option<u8> = if env
      .is_same_object(&volume, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&volume, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
    let pitch: Option<u8> = if env
      .is_same_object(&pitch, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .call_method(&pitch, "byteValue", "()B", &[])
          .map_err(OutputError::into_unknown)?
          .b()
          .map_err(OutputError::into_unknown)? as u8,
      )
    };
    let braille_backend: Option<String> = if env
      .is_same_object(&braille_backend, &null)
      .map_err(OutputError::into_unknown)?
    {
      None
    } else {
      Some(
        env
          .get_string(&braille_backend)
          .map_err(OutputError::into_unknown)?
          .into(),
      )
    };
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
  match closure() {
    Ok(()) => (),
    Err(OutputError::Unknown(error)) => match error.downcast_ref::<Error>() {
      Some(Error::JavaException) => (),
      _ => {
        let error = OutputError::into_unknown(error);
        let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
      }
    },
    Err(error) => {
      let _ = env.throw_new(error_to_exception_class(&error), error.to_string());
    }
  }
}
