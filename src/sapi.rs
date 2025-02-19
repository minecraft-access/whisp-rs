use sapi_lite::{audio::{AudioFormat,AudioStream,BitRate,Channels,MemoryStream,SampleRate},initialize,tts::{installed_voices,SpeechBuilder,SpeechOutput,SyncSynthesizer,VoiceSelector}};
use std::ffi::c_void;
use windows::Win32::System::Com::{IStream,STREAM_SEEK_SET};
use std::fmt;
use crate::speech_synthesizer::{SpeechError,SpeechResult,SpeechSynthesizer,Voice};
pub struct Sapi {
  synthesizer: SyncSynthesizer
}
impl fmt::Debug for Sapi {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Sapi").finish()
  }
}
impl SpeechSynthesizer for Sapi {
  fn new() -> Result<Self, SpeechError> {
    initialize()?;
    Ok(Sapi { synthesizer: SyncSynthesizer::new()? })
  }
  fn name(&self) -> String {
    "SAPI 5".to_owned()
  }
  fn min_rate(&self) -> u32 {
    0
  }
  fn max_rate(&self) -> u32 {
    100
  }
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError> {
    let voices = installed_voices(None, None)?;
    let voices = voices.filter_map(|voice| {
      match (voice.name(), voice.language()) {
        (None, _) => None,
        (_, None) => None,
        (Some(name), Some(language)) => Some(Voice { synthesizer: self, display_name: name.clone().into_string().ok()?, name: name.into_string().ok()?, language: language.into_string().ok()? }),
      }
    }).collect::<Vec<Voice>>();
    Ok(voices)
  }
  fn speak(&self, voice: &str, rate: u32, volume: u8, pitch: u8, text: &str) -> Result<SpeechResult, SpeechError> {
    let voice = installed_voices(Some(VoiceSelector::new().name_eq(voice)), None)?.next().ok_or(SpeechError { message: "No SAPI voices found with this name".to_owned() })?;
    self.synthesizer.set_voice(&voice)?;
    let rate: i32 = (rate/5-10).try_into()?;
    self.synthesizer.set_rate(rate)?;
    self.synthesizer.set_volume::<u32>(volume.into())?;
    let memory_stream = MemoryStream::new(None)?;
    let audio_format = AudioFormat { sample_rate: SampleRate::Hz44100, bit_rate: BitRate::Bits16, channels: Channels::Mono };
    let audio_stream = AudioStream::from_stream(memory_stream.try_clone()?, &audio_format)?;
    self.synthesizer.set_output(SpeechOutput::Stream(audio_stream), false)?;
    let pitch: i32 = (pitch/5-10).try_into()?;
    let speech = SpeechBuilder::new()
      .start_pitch(pitch)
      .say(text)
      .build();
    self.synthesizer.speak(speech, None)?;
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
    Ok(SpeechResult { pcm, sample_rate: 44100 })
  }
}
