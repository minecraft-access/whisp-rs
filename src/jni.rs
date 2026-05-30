#![allow(clippy::needless_pass_by_value)]
use crate::error::OutputError;
use crate::{SpeechSynthesizerMetadata, Whisprs};
use ::jni::errors::{Error, ErrorPolicy};
use ::jni::objects::{JClass, JObjectArray, JString};
use ::jni::refs::Reference as _;
use ::jni::strings::JNIString;
use ::jni::sys::{jboolean, jlong, JNI_FALSE};
use ::jni::{bind_java_type, jni_str, native_method, Env, JNIVersion, JavaVM, NativeMethod};
use anyhow::anyhow;
impl From<Error> for OutputError {
  fn from(error: Error) -> Self {
    OutputError::into_unknown(error)
  }
}
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
      OutputError::InvalidParameter(_) => "InvalidParameterException",
      OutputError::Unknown(_) => "UnknownException",
    }
}
struct ThrowOutputError;
impl<T: Default> ErrorPolicy<T, OutputError> for ThrowOutputError {
  type Captures<'unowned_env_local: 'native_method, 'native_method> = ();
  fn on_error<'unowned_env_local: 'native_method, 'native_method>(
    env: &mut Env<'unowned_env_local>,
    _cap: &mut Self::Captures<'unowned_env_local, 'native_method>,
    err: OutputError,
  ) -> ::jni::errors::Result<T> {
    if env.exception_check() {
      return Ok(T::default());
    }
    let class = JNIString::from(error_to_exception_class(&err));
    let message = JNIString::from(err.to_string());
    let _ = env.throw_new(class, message);
    Ok(T::default())
  }
  fn on_panic<'unowned_env_local: 'native_method, 'native_method>(
    env: &mut Env<'unowned_env_local>,
    _cap: &mut Self::Captures<'unowned_env_local, 'native_method>,
    payload: Box<dyn std::any::Any + Send + 'static>,
  ) -> ::jni::errors::Result<T> {
    if env.exception_check() {
      return Ok(T::default());
    }
    let message = payload
      .downcast_ref::<&str>()
      .map(|message| (*message).to_owned())
      .or_else(|| payload.downcast_ref::<String>().cloned())
      .unwrap_or_else(|| "Rust panic in native method".to_owned());
    let _ = env.throw_new(
      jni_str!("org/mcaccess/whisprs/error/UnknownException"),
      JNIString::from(message),
    );
    Ok(T::default())
  }
}
bind_java_type! {
  SpeechSynthesizerMetadataRef => org.mcaccess.whisprs.metadata.SpeechSynthesizerMetadata,
  constructors {
    fn new(
      name: JString,
      supports_speaking_to_audio_data: jboolean,
      supports_speech_parameters: jboolean,
    ),
  },
}
bind_java_type! {
  VoiceRef => org.mcaccess.whisprs.metadata.Voice,
  type_map = {
    SpeechSynthesizerMetadataRef => org.mcaccess.whisprs.metadata.SpeechSynthesizerMetadata,
  },
  constructors {
    fn new(
      synthesizer: SpeechSynthesizerMetadataRef,
      display_name: JString,
      name: JString,
      languages: JString[],
      priority: jbyte,
    ),
  },
}
bind_java_type! {
  BrailleBackendMetadataRef => org.mcaccess.whisprs.metadata.BrailleBackendMetadata,
  constructors {
    fn new(name: JString, priority: jbyte),
  },
}
bind_java_type! {
  SpeechResultRef => org.mcaccess.whisprs.audio.SpeechResult,
  constructors {
    fn new(pcm: jbyte[], sample_format: jbyte, sample_rate: jint),
  },
}
bind_java_type! {
  WhisprsRef => org.mcaccess.whisprs.Whisprs,
  fields {
    handle: jlong,
  },
}
bind_java_type! {
  JavaByteRef => java.lang.Byte,
  methods {
    fn byte_value() -> jbyte,
  },
}
fn jni_optional_string_to_rust(
  env: &mut Env,
  string: &JString,
) -> Result<Option<String>, OutputError> {
  if string.is_null() {
    Ok(None)
  } else {
    Ok(Some(string.try_to_string(env)?))
  }
}
fn jni_string_to_rust(env: &mut Env, string: &JString) -> Result<String, OutputError> {
  if string.is_null() {
    Err(OutputError::into_invalid_parameter(anyhow!(
      "Non-optional string parameter is null"
    )))
  } else {
    Ok(string.try_to_string(env)?)
  }
}
#[allow(clippy::cast_sign_loss)]
fn jni_optional_byte_to_rust(
  env: &mut Env,
  byte: &JavaByteRef,
) -> Result<Option<u8>, OutputError> {
  if byte.is_null() {
    Ok(None)
  } else {
    Ok(Some(byte.byte_value(env)? as u8))
  }
}
fn speech_synthesizer_metadata_to_jni<'local>(
  env: &mut Env<'local>,
  synthesizer: &SpeechSynthesizerMetadata,
) -> Result<SpeechSynthesizerMetadataRef<'local>, OutputError> {
  let name = env.new_string(&synthesizer.name)?;
  Ok(SpeechSynthesizerMetadataRef::new(
    env,
    &name,
    jboolean::from(synthesizer.supports_speaking_to_audio_data),
    jboolean::from(synthesizer.supports_speech_parameters),
  )?)
}
fn whisprs_from_handle<'a>(env: &mut Env, this: &WhisprsRef) -> Result<&'a Whisprs, OutputError> {
  let handle = this.handle(env)?;
  let whisprs = handle as *mut Whisprs;
  unsafe { whisprs.as_ref() }
    .ok_or_else(|| OutputError::into_invalid_parameter(anyhow!("Whisprs handle is null")))
}
const CREATE: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_create",
  static fn create() -> jlong,
};
fn create<'local>(_env: &mut Env<'local>, _class: JClass<'local>) -> Result<jlong, OutputError> {
  let whisprs = Whisprs::new()?;
  Ok(Box::into_raw(Box::new(whisprs)) as jlong)
}
const DESTROY: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_destroy",
  static fn destroy(handle: jlong),
};
#[allow(clippy::unnecessary_wraps)]
fn destroy<'local>(
  _env: &mut Env<'local>,
  _class: JClass<'local>,
  handle: jlong,
) -> Result<(), OutputError> {
  let whisprs = handle as *mut Whisprs;
  if !whisprs.is_null() {
    let _whisprs = unsafe { Box::from_raw(whisprs) };
  }
  Ok(())
}
const LIST_VOICES: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_listVoices",
  type_map = {
    VoiceRef => org.mcaccess.whisprs.metadata.Voice,
  },
  fn list_voices(
    synthesizer: JString,
    name: JString,
    language: JString,
    needs_audio_data: jboolean,
  ) -> VoiceRef[],
};
#[allow(clippy::cast_possible_wrap)]
fn list_voices<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
  synthesizer: JString<'local>,
  name: JString<'local>,
  language: JString<'local>,
  needs_audio_data: jboolean,
) -> Result<JObjectArray<'local, VoiceRef<'local>>, OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let synthesizer = jni_optional_string_to_rust(env, &synthesizer)?;
  let name = jni_optional_string_to_rust(env, &name)?;
  let language = jni_optional_string_to_rust(env, &language)?;
  let needs_audio_data: bool = needs_audio_data != JNI_FALSE;
  let voices = whisprs.list_voices(
    synthesizer.as_deref(),
    name.as_deref(),
    language.as_deref(),
    needs_audio_data,
  )?;
  let mut voice_refs: Vec<VoiceRef<'local>> = Vec::with_capacity(voices.len());
  for voice in voices {
    let synthesizer = speech_synthesizer_metadata_to_jni(env, &voice.synthesizer)?;
    let display_name = env.new_string(&voice.display_name)?;
    let name = env.new_string(&voice.name)?;
    let mut languages: Vec<JString<'local>> = Vec::with_capacity(voice.languages.len());
    for language in voice.languages {
      languages.push(env.new_string(language)?);
    }
    let languages_array = JObjectArray::<JString>::new(env, languages.len(), &JString::null())?;
    for (index, language) in languages.iter().enumerate() {
      languages_array.set_element(env, index, language)?;
    }
    let priority = voice.priority as i8;
    voice_refs.push(VoiceRef::new(
      env,
      &synthesizer,
      &display_name,
      &name,
      &languages_array,
      priority,
    )?);
  }
  let array = JObjectArray::<VoiceRef>::new(env, voice_refs.len(), &VoiceRef::null())?;
  for (index, voice) in voice_refs.iter().enumerate() {
    array.set_element(env, index, voice)?;
  }
  Ok(array)
}
const LIST_SPEECH_SYNTHESIZERS: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_listSpeechSynthesizers",
  type_map = {
    SpeechSynthesizerMetadataRef => org.mcaccess.whisprs.metadata.SpeechSynthesizerMetadata,
  },
  fn list_speech_synthesizers() -> SpeechSynthesizerMetadataRef[],
};
fn list_speech_synthesizers<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
) -> Result<JObjectArray<'local, SpeechSynthesizerMetadataRef<'local>>, OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let synthesizers = whisprs.list_speech_synthesizers()?;
  let mut refs: Vec<SpeechSynthesizerMetadataRef<'local>> = Vec::with_capacity(synthesizers.len());
  for synthesizer in &synthesizers {
    refs.push(speech_synthesizer_metadata_to_jni(env, synthesizer)?);
  }
  let array =
    JObjectArray::<SpeechSynthesizerMetadataRef>::new(env, refs.len(), &SpeechSynthesizerMetadataRef::null())?;
  for (index, synthesizer) in refs.iter().enumerate() {
    array.set_element(env, index, synthesizer)?;
  }
  Ok(array)
}
const LIST_SPEECH_SYNTHESIZERS_SUPPORTING_AUDIO_DATA: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_listSpeechSynthesizersSupportingAudioData",
  type_map = {
    SpeechSynthesizerMetadataRef => org.mcaccess.whisprs.metadata.SpeechSynthesizerMetadata,
  },
  fn list_speech_synthesizers_supporting_audio_data() -> SpeechSynthesizerMetadataRef[],
};
fn list_speech_synthesizers_supporting_audio_data<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
) -> Result<JObjectArray<'local, SpeechSynthesizerMetadataRef<'local>>, OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let synthesizers = whisprs.list_speech_synthesizers_supporting_audio_data()?;
  let mut refs: Vec<SpeechSynthesizerMetadataRef<'local>> = Vec::with_capacity(synthesizers.len());
  for synthesizer in &synthesizers {
    refs.push(speech_synthesizer_metadata_to_jni(env, synthesizer)?);
  }
  let array =
    JObjectArray::<SpeechSynthesizerMetadataRef>::new(env, refs.len(), &SpeechSynthesizerMetadataRef::null())?;
  for (index, synthesizer) in refs.iter().enumerate() {
    array.set_element(env, index, synthesizer)?;
  }
  Ok(array)
}
const LIST_BRAILLE_BACKENDS: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_listBrailleBackends",
  type_map = {
    BrailleBackendMetadataRef => org.mcaccess.whisprs.metadata.BrailleBackendMetadata,
  },
  fn list_braille_backends() -> BrailleBackendMetadataRef[],
};
#[allow(clippy::cast_possible_wrap)]
fn list_braille_backends<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
) -> Result<JObjectArray<'local, BrailleBackendMetadataRef<'local>>, OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let backends = whisprs.list_braille_backends()?;
  let mut refs: Vec<BrailleBackendMetadataRef<'local>> = Vec::with_capacity(backends.len());
  for backend in backends {
    let name = env.new_string(&backend.name)?;
    let priority = backend.priority as i8;
    refs.push(BrailleBackendMetadataRef::new(env, &name, priority)?);
  }
  let array =
    JObjectArray::<BrailleBackendMetadataRef>::new(env, refs.len(), &BrailleBackendMetadataRef::null())?;
  for (index, backend) in refs.iter().enumerate() {
    array.set_element(env, index, backend)?;
  }
  Ok(array)
}
const SPEAK_TO_AUDIO_DATA: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_speakToAudioData",
  type_map = {
    SpeechResultRef => org.mcaccess.whisprs.audio.SpeechResult,
    JavaByteRef => java.lang.Byte,
  },
  fn speak_to_audio_data(
    synthesizer: JString,
    voice: JString,
    language: JString,
    rate: JavaByteRef,
    volume: JavaByteRef,
    pitch: JavaByteRef,
    text: JString,
  ) -> SpeechResultRef,
};
#[allow(clippy::too_many_arguments)]
fn speak_to_audio_data<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
  synthesizer: JString<'local>,
  voice: JString<'local>,
  language: JString<'local>,
  rate: JavaByteRef<'local>,
  volume: JavaByteRef<'local>,
  pitch: JavaByteRef<'local>,
  text: JString<'local>,
) -> Result<SpeechResultRef<'local>, OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let synthesizer = jni_optional_string_to_rust(env, &synthesizer)?;
  let voice = jni_optional_string_to_rust(env, &voice)?;
  let language = jni_optional_string_to_rust(env, &language)?;
  let rate = jni_optional_byte_to_rust(env, &rate)?;
  let volume = jni_optional_byte_to_rust(env, &volume)?;
  let pitch = jni_optional_byte_to_rust(env, &pitch)?;
  let text = jni_string_to_rust(env, &text)?;
  let result = whisprs.speak_to_audio_data(
    synthesizer.as_deref(),
    voice.as_deref(),
    language.as_deref(),
    rate,
    volume,
    pitch,
    &text,
  )?;
  let buffer = env.byte_array_from_slice(&result.pcm)?;
  #[allow(clippy::cast_possible_wrap)]
  let sample_format = result.sample_format as i8;
  let sample_rate = result
    .sample_rate
    .try_into()
    .map_err(OutputError::into_unknown)?;
  Ok(SpeechResultRef::new(
    env,
    &buffer,
    sample_format,
    sample_rate,
  )?)
}
const SPEAK_TO_AUDIO_OUTPUT: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_speakToAudioOutput",
  type_map = {
    JavaByteRef => java.lang.Byte,
  },
  fn speak_to_audio_output(
    synthesizer: JString,
    voice: JString,
    language: JString,
    rate: JavaByteRef,
    volume: JavaByteRef,
    pitch: JavaByteRef,
    text: JString,
    interrupt: jboolean,
  ),
};
#[allow(clippy::too_many_arguments)]
fn speak_to_audio_output<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
  synthesizer: JString<'local>,
  voice: JString<'local>,
  language: JString<'local>,
  rate: JavaByteRef<'local>,
  volume: JavaByteRef<'local>,
  pitch: JavaByteRef<'local>,
  text: JString<'local>,
  interrupt: jboolean,
) -> Result<(), OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let synthesizer = jni_optional_string_to_rust(env, &synthesizer)?;
  let voice = jni_optional_string_to_rust(env, &voice)?;
  let language = jni_optional_string_to_rust(env, &language)?;
  let rate = jni_optional_byte_to_rust(env, &rate)?;
  let volume = jni_optional_byte_to_rust(env, &volume)?;
  let pitch = jni_optional_byte_to_rust(env, &pitch)?;
  let text = jni_string_to_rust(env, &text)?;
  let interrupt: bool = interrupt != JNI_FALSE;
  whisprs.speak_to_audio_output(
    synthesizer.as_deref(),
    voice.as_deref(),
    language.as_deref(),
    rate,
    volume,
    pitch,
    &text,
    interrupt,
  )
}
const STOP_SPEECH: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_stopSpeech",
  fn stop_speech(synthesizer: JString),
};
fn stop_speech<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
  synthesizer: JString<'local>,
) -> Result<(), OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let synthesizer = jni_optional_string_to_rust(env, &synthesizer)?;
  whisprs.stop_speech(synthesizer.as_deref())
}
const BRAILLE: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_braille",
  fn braille(backend: JString, text: JString),
};
fn braille<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
  backend: JString<'local>,
  text: JString<'local>,
) -> Result<(), OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let backend = jni_optional_string_to_rust(env, &backend)?;
  let text = jni_string_to_rust(env, &text)?;
  whisprs.braille(backend.as_deref(), &text)
}
const OUTPUT: NativeMethod = native_method! {
  java_type = "org.mcaccess.whisprs.Whisprs",
  rust_type = WhisprsRef,
  error_policy = ThrowOutputError,
  export = "Java_org_mcaccess_whisprs_Whisprs_output",
  type_map = {
    JavaByteRef => java.lang.Byte,
  },
  fn output(
    synthesizer: JString,
    voice: JString,
    language: JString,
    rate: JavaByteRef,
    volume: JavaByteRef,
    pitch: JavaByteRef,
    braille_backend: JString,
    text: JString,
    interrupt: jboolean,
  ),
};
#[allow(clippy::too_many_arguments)]
fn output<'local>(
  env: &mut Env<'local>,
  this: WhisprsRef<'local>,
  synthesizer: JString<'local>,
  voice: JString<'local>,
  language: JString<'local>,
  rate: JavaByteRef<'local>,
  volume: JavaByteRef<'local>,
  pitch: JavaByteRef<'local>,
  braille_backend: JString<'local>,
  text: JString<'local>,
  interrupt: jboolean,
) -> Result<(), OutputError> {
  let whisprs = whisprs_from_handle(env, &this)?;
  let synthesizer = jni_optional_string_to_rust(env, &synthesizer)?;
  let voice = jni_optional_string_to_rust(env, &voice)?;
  let language = jni_optional_string_to_rust(env, &language)?;
  let rate = jni_optional_byte_to_rust(env, &rate)?;
  let volume = jni_optional_byte_to_rust(env, &volume)?;
  let pitch = jni_optional_byte_to_rust(env, &pitch)?;
  let braille_backend = jni_optional_string_to_rust(env, &braille_backend)?;
  let text = jni_string_to_rust(env, &text)?;
  let interrupt: bool = interrupt != JNI_FALSE;
  whisprs.output(
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
}
const NATIVE_METHODS: &[NativeMethod] = &[
  CREATE,
  DESTROY,
  LIST_VOICES,
  LIST_SPEECH_SYNTHESIZERS,
  LIST_SPEECH_SYNTHESIZERS_SUPPORTING_AUDIO_DATA,
  LIST_BRAILLE_BACKENDS,
  SPEAK_TO_AUDIO_DATA,
  SPEAK_TO_AUDIO_OUTPUT,
  STOP_SPEECH,
  BRAILLE,
  OUTPUT,
];
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn JNI_OnLoad(
  vm: *mut ::jni::sys::JavaVM,
  _reserved: *mut std::ffi::c_void,
) -> ::jni::sys::jint {
  let vm = unsafe { JavaVM::from_raw(vm) };
  let _ = vm.attach_current_thread(|env| {
    unsafe { env.register_native_methods(jni_str!("org/mcaccess/whisprs/Whisprs"), NATIVE_METHODS) }
  });
  JNIVersion::V1_6.into()
}
