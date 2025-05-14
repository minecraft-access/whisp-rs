use std::cell::{OnceCell,RefCell};
use std::collections::HashMap;
use std::sync::{mpsc,Mutex,OnceLock};
use std::thread;
use crate::speech_synthesizer::*;
use crate::espeak_ng::EspeakNg;
#[cfg(windows)] use crate::sapi::Sapi;
#[cfg(target_os = "macos")] use crate::av_speech_synthesizer::AvSpeechSynthesizer;
thread_local! {
  static SYNTHESIZERS: RefCell<HashMap<String, Box<dyn SpeechSynthesizer>>> = RefCell::new(HashMap::new());
}
enum Operation {
  ListVoices,
  SpeakToAudioData(String, String, String, Option<u8>, Option<u8>, Option<u8>, String),
  SpeakToAudioOutput(String, String, String, Option<u8>, Option<u8>, Option<u8>, String, bool),
  StopSpeech(String)
}
enum ResultValue {
  Initialize(Result<(), SpeechError>),
  ListVoices(Result<Vec<Voice>, SpeechError>),
  SpeakToAudioData(Result<SpeechResult, SpeechError>),
  SpeakToAudioOutput(Result<(), SpeechError>),
  StopSpeech(Result<(), SpeechError>)
}
static OPERATION_TX: OnceLock<mpsc::Sender<Operation>> = OnceLock::new();
static RESULT_RX: Mutex<OnceCell<mpsc::Receiver<ResultValue>>> = Mutex::new(OnceCell::new());
pub fn initialize() -> Result<(), SpeechError> {
  let (operation_tx, operation_rx) = mpsc::channel();
  OPERATION_TX.set(operation_tx).unwrap();
  let (result_tx, result_rx) = mpsc::channel();
  RESULT_RX.lock().unwrap().set(result_rx).unwrap();
  thread::spawn(move || {
    let result = SYNTHESIZERS.with_borrow_mut(|synthesizers| {
      let espeak_ng = EspeakNg::new()?;
      synthesizers.insert(espeak_ng.data().name, Box::new(espeak_ng));
      #[cfg(windows)] {
        let sapi = Sapi::new()?;
        synthesizers.insert(sapi.data().name, Box::new(sapi));
      }
      #[cfg(target_os = "macos")] {
        let av_speech_synthesizer = AvSpeechSynthesizer::new()?;
        synthesizers.insert(av_speech_synthesizer.data().name, Box::new(av_speech_synthesizer));
      }
      Ok(())
    });
    result_tx.send(ResultValue::Initialize(result)).unwrap();
    while let Ok(operation) = operation_rx.recv() {
      match operation {
        Operation::ListVoices => result_tx.send(ResultValue::ListVoices(internal_list_voices())).unwrap(),
        Operation::SpeakToAudioData(synthesizer, voice, language, rate, volume, pitch, text) => result_tx.send(ResultValue::SpeakToAudioData(internal_speak_to_audio_data(&synthesizer, &voice, &language, rate, volume, pitch, &text))).unwrap(),
        Operation::SpeakToAudioOutput(synthesizer, voice, language, rate, volume, pitch, text, interrupt) => result_tx.send(ResultValue::SpeakToAudioOutput(internal_speak_to_audio_output(&synthesizer, &voice, &language, rate, volume, pitch, &text, interrupt))).unwrap(),
        Operation::StopSpeech(synthesizer) => result_tx.send(ResultValue::StopSpeech(internal_stop_speech(&synthesizer))).unwrap()
      };
    };
  });
  match RESULT_RX.lock()?.get().unwrap().recv().unwrap() {
    ResultValue::Initialize(result) => result,
    _ => panic!("Invalid initialization result")
  }
}
fn internal_list_voices() -> Result<Vec<Voice>, SpeechError> {
  SYNTHESIZERS.with_borrow(|synthesizers| {
    let voices = synthesizers.values()
      .map(|synthesizer| synthesizer.list_voices())
      .collect::<Result<Vec<Vec<Voice>>, SpeechError>>()?;
    Ok(voices.into_iter().flatten().collect::<Vec<Voice>>())
  })
}
pub fn list_voices() -> Result<Vec<Voice>, SpeechError> {
  OPERATION_TX.get().unwrap().send(Operation::ListVoices)?;
  match RESULT_RX.lock()?.get().unwrap().recv()? {
    ResultValue::ListVoices(result) => result,
    _ => panic!("Received result value for other operation")
  }
}
fn internal_speak_to_audio_data(synthesizer: &str, voice: &str, language: &str, rate: Option<u8>, volume: Option<u8>, pitch: Option<u8>, text: &str) -> Result<SpeechResult, SpeechError> {
  SYNTHESIZERS.with_borrow(|synthesizers| {
    match synthesizers.get(synthesizer) {
      None => return Err(SpeechError { message: "Unknown synthesizer".to_owned() }),
      Some(synthesizer) => match synthesizer.as_to_audio_data() {
        None => return Err(SpeechError { message: "Synthesizer does not support returning audio data".to_owned() }),
        Some(synthesizer) => synthesizer.speak(voice, language, rate, volume, pitch, text)
      }
    }
  })
}
pub fn speak_to_audio_data(synthesizer: &str, voice: &str, language: &str, rate: Option<u8>, volume: Option<u8>, pitch: Option<u8>, text: &str) -> Result<SpeechResult, SpeechError> {
  OPERATION_TX.get().unwrap().send(Operation::SpeakToAudioData(synthesizer.to_owned(), voice.to_owned(), language.to_owned(), rate, volume, pitch, text.to_owned()))?;
  match RESULT_RX.lock()?.get().unwrap().recv()? {
    ResultValue::SpeakToAudioData(result) => result,
    _ => panic!("Received result value for other operation")
  }
}
fn internal_speak_to_audio_output(synthesizer: &str, voice: &str, language: &str, rate: Option<u8>, volume: Option<u8>, pitch: Option<u8>, text: &str, interrupt: bool) -> Result<(), SpeechError> {
  SYNTHESIZERS.with_borrow(|synthesizers| {
    match synthesizers.get(synthesizer) {
      None => return Err(SpeechError { message: "Unknown synthesizer".to_owned() }),
      Some(synthesizer) => match synthesizer.as_to_audio_output() {
        None => return Err(SpeechError { message: "Synthesizer does not support speaking to audio output".to_owned() }),
        Some(synthesizer) => synthesizer.speak(voice, language, rate, volume, pitch, text, interrupt)
      }
    }
  })
}
pub fn speak_to_audio_output(synthesizer: &str, voice: &str, language: &str, rate: Option<u8>, volume: Option<u8>, pitch: Option<u8>, text: &str, interrupt: bool) -> Result<(), SpeechError> {
  OPERATION_TX.get().unwrap().send(Operation::SpeakToAudioOutput(synthesizer.to_owned(), voice.to_owned(), language.to_owned(), rate, volume, pitch, text.to_owned(), interrupt))?;
  match RESULT_RX.lock()?.get().unwrap().recv()? {
    ResultValue::SpeakToAudioOutput(result) => result,
    _ => panic!("Received result value for other operation")
  }
}
fn internal_stop_speech(synthesizer: &str) -> Result<(), SpeechError> {
  SYNTHESIZERS.with_borrow(|synthesizers| {
    match synthesizers.get(synthesizer) {
      None => return Err(SpeechError { message: "Unknown synthesizer".to_owned() }),
      Some(synthesizer) => match synthesizer.as_to_audio_output() {
        None => return Err(SpeechError { message: "Synthesizer does not support speaking to audio output".to_owned() }),
        Some(synthesizer) => synthesizer.stop_speech()
      }
    }
  })
}
pub fn stop_speech(synthesizer: &str) -> Result<(), SpeechError> {
  OPERATION_TX.get().unwrap().send(Operation::StopSpeech(synthesizer.to_owned()))?;
  match RESULT_RX.lock()?.get().unwrap().recv()? {
    ResultValue::StopSpeech(result) => result,
    _ => panic!("Received result value for other operation")
  }
}
