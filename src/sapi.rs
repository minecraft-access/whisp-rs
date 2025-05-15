use std::ffi::c_void;
use windows::core::*;
use windows::Win32::Media::Speech::*;
use windows::Win32::System::Com::*;
use crate::speech_synthesizer::*;
pub struct Sapi {
  synthesizer: ISpVoice
}
impl SpeechSynthesizer for Sapi {
  fn new() -> std::result::Result<Self, SpeechError> {
    unsafe {
      let synthesizer: ISpVoice = CoCreateInstance(&SpVoice, None, CLSCTX_ALL).unwrap();
      Ok(Sapi { synthesizer })
    }
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData { name: "SAPI 5".to_owned(), supports_to_audio_data: false, supports_to_audio_output: false, supports_speech_parameters: false }
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, SpeechError> {
    unsafe {
      let category: ISpObjectTokenCategory = CoCreateInstance(&SpObjectTokenCategory, None, CLSCTX_ALL).unwrap();
      category.SetId(SPCAT_VOICES, false).unwrap();
      let enumerator = category.EnumTokens(None, None).unwrap();
      let mut count: u32 = 0;
      enumerator.GetCount(&mut count).unwrap();
      let mut tokens = Vec::with_capacity(count.try_into().unwrap());
      let mut tokens_fetched: u32 = 0;
      enumerator.Next(count, tokens.as_mut_ptr(), Some(&mut tokens_fetched)).unwrap();
      tokens.set_len(tokens_fetched.try_into().unwrap());
      let voices = tokens
        .into_iter()
        .filter(|option| option.is_some())
        .map(|token| {
          let token = token.as_ref().unwrap();
          let name = token.GetId().unwrap().to_string().unwrap();
          let display_name = token.GetStringValue(w!("Name"));
          let language = token.GetStringValue(w!("Language"));
          let display_name = match display_name {
            Ok(display_name) => display_name.to_string().unwrap(),
            _ => "Unknown".to_owned()
          };
          let languages = match language {
            Ok(language) => vec!(language.to_string().unwrap()),
            _ => vec!()
          };
          Voice { synthesizer: self.data(), display_name, name, languages, priority: 2 }
        })
        .collect::<Vec<Voice>>();
      Ok(voices)
    }
  }
  fn as_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    None
  }
  fn as_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    None
  }
}
/*
  fn speak(&self, voice: &str, language: &str, rate: u8, volume: u8, pitch: u8, text: &str) -> std::result::Result<SpeechResult, SpeechError> {
    let voice = installed_voices(Some(VoiceSelector::new().name_eq(voice)), None)?
      .filter(|voice| {
        match voice.language() {
          None => language=="none",
          Some(os_string) => os_string.into_string().unwrap().to_lowercase()==language
        }
      })
      .next()
      .ok_or(SpeechError { message: "No SAPI voices found with this name and language".to_owned() })?;
    synthesizer.set_voice(&voice)?;
    let rate = rate as i32;
    let rate = (rate/5)-10;
    synthesizer.set_rate(rate)?;
    synthesizer.set_volume(volume as u32)?;
    let memory_stream = MemoryStream::new(None)?;
    let audio_format = AudioFormat { sample_rate: SampleRate::Hz44100, bit_rate: BitRate::Bits16, channels: Channels::Mono };
    let audio_stream = AudioStream::from_stream(memory_stream.try_clone()?, &audio_format)?;
    synthesizer.set_output(SpeechOutput::Stream(audio_stream), false)?;
    let pitch = pitch as i32;
    let pitch = (pitch/5)-10;
    let speech = SpeechBuilder::new()
      .start_pitch(pitch)
      .say(text)
      .build();
    synthesizer.speak(speech, None)?;
    let mut pcm: Vec<u8> = Vec::new();
    let mut buffer: Vec<u8> = Vec::with_capacity(65536);
    let mut bytes_read: u32 = 0;
    let stream: IStream = memory_stream.into();
    unsafe { stream.Seek(0, STREAM_SEEK_SET)? };
    loop {
      let result = unsafe { stream.Read(buffer.as_mut_ptr() as *mut c_void, 65536, &mut bytes_read) };
      if bytes_read==0 {
        break
      }
      unsafe { buffer.set_len(bytes_read.try_into()?) };
      pcm.append(&mut buffer);
      buffer.clear();
      match result {
        Ok(()) => {},
        Err(_) => break
      };
    }
    Ok(SpeechResult { pcm, sample_format: SampleFormat::S16, sample_rate: 44100 })
  }
*/
