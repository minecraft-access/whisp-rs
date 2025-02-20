use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use crate::speech_synthesizer::{SpeechError,SpeechResult,SpeechSynthesizer,Voice};
use crate::espeak_ng::EspeakNg;
#[cfg(windows)] use crate::sapi::Sapi;
lazy_static! {
  static ref SYNTHESIZERS: Mutex<HashMap<String, Box<(dyn SpeechSynthesizer + Send)>>> = Mutex::new(HashMap::new());
}
pub fn initialize() -> Result<(), SpeechError> {
  let mut synthesizers = SYNTHESIZERS.lock()?;
  let espeak_ng = EspeakNg::new()?;
  synthesizers.insert(espeak_ng.name(), Box::new(espeak_ng));
  #[cfg(windows)] {
    let sapi = Sapi::new()?;
    synthesizers.insert(sapi.name(), Box::new(sapi));
  }
  Ok(())
}
pub fn list_voices() -> Result<Vec<Voice>, SpeechError> {
  let synthesizers = SYNTHESIZERS.lock()?;
  let voices = synthesizers.values()
    .map(|synthesizer| synthesizer.list_voices())
    .collect::<Result<Vec<Vec<Voice>>, SpeechError>>()?;
  Ok(voices.into_iter().flatten().collect::<Vec<Voice>>())
}
pub fn speak(synthesizer: &str, voice: &str, language: &str, rate: u32, volume: u8, pitch: u8, text: &str) -> Result<SpeechResult, SpeechError> {
  match SYNTHESIZERS.lock()?.get(synthesizer) {
    None => return Err(SpeechError { message: "Unknown synthesizer".to_owned() }),
    Some(synthesizer) => synthesizer.speak(voice, language, rate, volume, pitch, text)
  }
}
