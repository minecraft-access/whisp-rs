use sapi_lite::{audio::{AudioFormat,AudioStream,BitRate,Channels,MemoryStream,SampleRate},initialize,tts::{installed_voices,SpeechBuilder,SpeechOutput,SyncSynthesizer,VoiceSelector}};
use std::ffi::c_void;
use windows::Win32::System::Com::{IStream,STREAM_SEEK_SET};
use std::fmt;
use std::sync::{mpsc,Mutex};
use std::thread;
use crate::speech_synthesizer::{SpeechError,SpeechResult,SpeechSynthesizer,Voice};
fn name() -> String {
  "SAPI 5".to_owned()
}
fn list_voices(synthesizer: String) -> Result<Vec<Voice>, SpeechError> {
  let voices = installed_voices(None, None)?;
  let voices = voices.filter_map(|voice| {
    match (voice.name(), voice.language()) {
      (None, _) => None,
      (Some(name), None) => Some(Voice { synthesizer: synthesizer.clone(), display_name: name.clone().into_string().ok()?, name: name.into_string().ok()?, language: "none".to_owned() }),
      (Some(name), Some(language)) => Some(Voice { synthesizer: synthesizer.clone(), display_name: name.clone().into_string().ok()?, name: name.into_string().ok()?, language: language.into_string().ok()?.to_lowercase() }),
    }
  }).collect::<Vec<Voice>>();
  Ok(voices)
}
fn speak(synthesizer: &SyncSynthesizer, voice: &str, rate: u32, volume: u8, pitch: u8, text: &str) -> Result<SpeechResult, SpeechError> {
  let voice = installed_voices(Some(VoiceSelector::new().name_eq(voice)), None)?.next().ok_or(SpeechError { message: "No SAPI voices found with this name".to_owned() })?;
  synthesizer.set_voice(&voice)?;
  let rate: i32 = rate.try_into()?;
  let rate: i32 = (rate/5)-10;
  synthesizer.set_rate(rate)?;
  synthesizer.set_volume::<u32>(volume.into())?;
  let memory_stream = MemoryStream::new(None)?;
  let audio_format = AudioFormat { sample_rate: SampleRate::Hz44100, bit_rate: BitRate::Bits16, channels: Channels::Mono };
  let audio_stream = AudioStream::from_stream(memory_stream.try_clone()?, &audio_format)?;
  synthesizer.set_output(SpeechOutput::Stream(audio_stream), false)?;
  let pitch: i32 = pitch.try_into()?;
  let pitch: i32 = (pitch/5)-10;
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
  Ok(SpeechResult { pcm, sample_rate: 44100 })
}
enum Operation {
  ListVoices,
  Speak(String, u32, u8, u8, String)
}
enum ResultValue {
  ListVoices(Result<Vec<Voice>, SpeechError>),
  Speak(Result<SpeechResult, SpeechError>)
}
pub struct Sapi {
  tx: mpsc::Sender<Operation>,
  rx: Mutex<mpsc::Receiver<ResultValue>>
}
impl fmt::Debug for Sapi {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Sapi").finish()
  }
}
impl SpeechSynthesizer for Sapi {
  fn new() -> Result<Self, SpeechError> {
    let (operation_tx, operation_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    thread::spawn(move || {
      initialize().unwrap();
      let synthesizer = SyncSynthesizer::new().unwrap();
      while let Ok(operation) = operation_rx.recv() {
        match operation {
          Operation::ListVoices => result_tx.send(ResultValue::ListVoices(list_voices(name()))).unwrap(),
          Operation::Speak(voice, rate, volume, pitch, text) => result_tx.send(ResultValue::Speak(speak(&synthesizer, &voice, rate, volume, pitch, &text))).unwrap()
        };
      };
    });
    Ok(Sapi { tx: operation_tx, rx: Mutex::new(result_rx) })
  }
  fn name(&self) -> String {
    name()
  }
  fn min_rate(&self) -> u32 {
    0
  }
  fn max_rate(&self) -> u32 {
    100
  }
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError> {
    self.tx.send(Operation::ListVoices)?;
    match self.rx.lock()?.recv()? {
      ResultValue::ListVoices(result) => result,
      _ => Err(SpeechError { message: "Received result value for other operation".to_owned() })
    }
  }
  fn speak(&self, voice: &str, rate: u32, volume: u8, pitch: u8, text: &str) -> Result<SpeechResult, SpeechError> {
    self.tx.send(Operation::Speak(voice.to_owned(), rate, volume, pitch, text.to_owned()))?;
    match self.rx.lock()?.recv()? {
      ResultValue::Speak(result) => result,
      _ => Err(SpeechError { message: "Received result value for other operation".to_owned() })
    }
  }
}
