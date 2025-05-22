#[cfg(target_os = "macos")]
use crate::av_speech_synthesizer::AvSpeechSynthesizer;
use crate::espeak_ng::EspeakNg;
#[cfg(windows)]
use crate::jaws::Jaws;
#[cfg(windows)]
use crate::nvda::Nvda;
#[cfg(windows)]
use crate::one_core::OneCore;
#[cfg(windows)]
use crate::sapi::Sapi;
#[cfg(target_os = "linux")]
use crate::speech_dispatcher::SpeechDispatcher;
use crate::speech_synthesizer::*;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
thread_local! {
  static SYNTHESIZERS: RefCell<HashMap<String, Box<dyn SpeechSynthesizer>>> = RefCell::new(HashMap::new());
  static OUTPUT_STREAM: OnceCell<Option<OutputStream>> = const {OnceCell::new() };
}
static SINK: OnceLock<Sink> = OnceLock::new();
enum Operation {
  ListVoices,
  SpeakToAudioData(
    Option<String>,
    Option<String>,
    Option<String>,
    Option<u8>,
    Option<u8>,
    Option<u8>,
    String,
  ),
  SpeakToAudioOutput(
    Option<String>,
    Option<String>,
    Option<String>,
    Option<u8>,
    Option<u8>,
    Option<u8>,
    String,
    bool,
  ),
  StopSpeech(String),
}
enum ResultValue {
  Initialize(Result<(), SpeechError>),
  ListVoices(Result<Vec<Voice>, SpeechError>),
  SpeakToAudioData(Result<SpeechResult, SpeechError>),
  SpeakToAudioOutput(Result<(), SpeechError>),
  StopSpeech(Result<(), SpeechError>),
}
static OPERATION_TX: OnceLock<mpsc::Sender<Operation>> = OnceLock::new();
static RESULT_RX: Mutex<OnceCell<mpsc::Receiver<ResultValue>>> = Mutex::new(OnceCell::new());
pub fn initialize() -> Result<(), SpeechError> {
  let (operation_tx, operation_rx) = mpsc::channel();
  OPERATION_TX.set(operation_tx).unwrap();
  let (result_tx, result_rx) = mpsc::channel();
  RESULT_RX.lock()?.set(result_rx).unwrap();
  thread::spawn(move || {
    let result = {
      let (output_stream, output_stream_handle) = OutputStream::try_default().unwrap();
      let sink = Sink::try_new(&output_stream_handle).unwrap();
      let _result = OUTPUT_STREAM.with(|cell| cell.set(Some(output_stream)));
      let _result = SINK.set(sink);
      let mut synthesizers: Vec<Result<Box<dyn SpeechSynthesizer>, SpeechError>> = Vec::new();
      synthesizers.push(EspeakNg::new().map(|value| Box::new(value) as Box<dyn SpeechSynthesizer>));
      #[cfg(windows)]
      {
        synthesizers.push(Sapi::new().map(|value| Box::new(value) as Box<dyn SpeechSynthesizer>));
        synthesizers
          .push(OneCore::new().map(|value| Box::new(value) as Box<dyn SpeechSynthesizer>));
        synthesizers.push(Jaws::new().map(|value| Box::new(value) as Box<dyn SpeechSynthesizer>));
        synthesizers.push(Nvda::new().map(|value| Box::new(value) as Box<dyn SpeechSynthesizer>));
      }
      #[cfg(target_os = "linux")]
      {
        synthesizers
          .push(SpeechDispatcher::new().map(|value| Box::new(value) as Box<dyn SpeechSynthesizer>));
      }
      #[cfg(target_os = "macos")]
      {
        synthesizers.push(
          AvSpeechSynthesizer::new().map(|value| Box::new(value) as Box<dyn SpeechSynthesizer>),
        );
      }
      SYNTHESIZERS.set(
        synthesizers
          .into_iter()
          .flatten()
          .map(|synthesizer| (synthesizer.data().name, synthesizer))
          .collect(),
      );
      Ok(())
    };
    result_tx.send(ResultValue::Initialize(result)).unwrap();
    while let Ok(operation) = operation_rx.recv() {
      match operation {
        Operation::ListVoices => result_tx
          .send(ResultValue::ListVoices(internal_list_voices()))
          .unwrap(),
        Operation::SpeakToAudioData(synthesizer, voice, language, rate, volume, pitch, text) => {
          result_tx
            .send(ResultValue::SpeakToAudioData(internal_speak_to_audio_data(
              synthesizer.as_deref(),
              voice.as_deref(),
              language.as_deref(),
              rate,
              volume,
              pitch,
              &text,
            )))
            .unwrap()
        }
        Operation::SpeakToAudioOutput(
          synthesizer,
          voice,
          language,
          rate,
          volume,
          pitch,
          text,
          interrupt,
        ) => result_tx
          .send(ResultValue::SpeakToAudioOutput(
            internal_speak_to_audio_output(
              synthesizer.as_deref(),
              voice.as_deref(),
              language.as_deref(),
              rate,
              volume,
              pitch,
              &text,
              interrupt,
            ),
          ))
          .unwrap(),
        Operation::StopSpeech(synthesizer) => result_tx
          .send(ResultValue::StopSpeech(internal_stop_speech(&synthesizer)))
          .unwrap(),
      };
    }
  });
  match RESULT_RX.lock()?.get().unwrap().recv().unwrap() {
    ResultValue::Initialize(result) => result,
    _ => panic!("Invalid initialization result"),
  }
}
fn internal_list_voices() -> Result<Vec<Voice>, SpeechError> {
  SYNTHESIZERS.with_borrow(|synthesizers| {
    let voices = synthesizers
      .values()
      .flat_map(|synthesizer| synthesizer.list_voices())
      .flatten()
      .collect::<Vec<Voice>>();
    Ok(voices)
  })
}
pub fn list_voices() -> Result<Vec<Voice>, SpeechError> {
  OPERATION_TX.get().unwrap().send(Operation::ListVoices)?;
  match RESULT_RX.lock()?.get().unwrap().recv()? {
    ResultValue::ListVoices(result) => result,
    _ => panic!("Received result value for other operation"),
  }
}
fn filter_synthesizers(
  synthesizer: Option<&str>,
  voice: Option<&str>,
  language: Option<&str>,
) -> Result<String, SpeechError> {
  let synthesizer = match (synthesizer, voice, language) {
    (Some(synthesizer), _, _) => synthesizer.to_owned(),
    (None, voice_name, language) => {
      let mut voices = internal_list_voices()?
        .into_iter()
        .filter(|voice| {
          voice_name.map(|name| voice.name == name).unwrap_or(true)
            || language
              .map(|name| voice.languages.iter().any(|language| language == name))
              .unwrap_or(true)
        })
        .collect::<Vec<Voice>>();
      voices.sort_unstable_by_key(|voice| voice.priority);
      voices
        .first()
        .ok_or(SpeechError {
          message: "Not voices found with given attributes".to_owned(),
        })?
        .synthesizer
        .name
        .clone()
    }
  };
  Ok(synthesizer)
}
fn internal_speak_to_audio_data(
  synthesizer: Option<&str>,
  voice: Option<&str>,
  language: Option<&str>,
  rate: Option<u8>,
  volume: Option<u8>,
  pitch: Option<u8>,
  text: &str,
) -> Result<SpeechResult, SpeechError> {
  SYNTHESIZERS.with_borrow(|synthesizers| {
    let synthesizer = filter_synthesizers(synthesizer, voice, language)?;
    match synthesizers.get(&synthesizer) {
      None => Err(SpeechError {
        message: "Unknown synthesizer".to_owned(),
      }),
      Some(synthesizer) => match synthesizer.as_to_audio_data() {
        None => Err(SpeechError {
          message: "Synthesizer does not support returning audio data".to_owned(),
        }),
        Some(synthesizer) => synthesizer.speak(voice, language, rate, volume, pitch, text),
      },
    }
  })
}
pub fn speak_to_audio_data(
  synthesizer: Option<&str>,
  voice: Option<&str>,
  language: Option<&str>,
  rate: Option<u8>,
  volume: Option<u8>,
  pitch: Option<u8>,
  text: &str,
) -> Result<SpeechResult, SpeechError> {
  OPERATION_TX
    .get()
    .unwrap()
    .send(Operation::SpeakToAudioData(
      synthesizer.map(|value| value.to_owned()),
      voice.map(|value| value.to_owned()),
      language.map(|value| value.to_owned()),
      rate,
      volume,
      pitch,
      text.to_owned(),
    ))?;
  match RESULT_RX.lock()?.get().unwrap().recv()? {
    ResultValue::SpeakToAudioData(result) => result,
    _ => panic!("Received result value for other operation"),
  }
}
fn internal_speak_to_audio_output(
  synthesizer: Option<&str>,
  voice: Option<&str>,
  language: Option<&str>,
  rate: Option<u8>,
  volume: Option<u8>,
  pitch: Option<u8>,
  text: &str,
  interrupt: bool,
) -> Result<(), SpeechError> {
  SYNTHESIZERS.with_borrow(|synthesizers| {
    let synthesizer = filter_synthesizers(synthesizer, voice, language)?;
    let synthesizer = match synthesizers.get(&synthesizer) {
      None => {
        return Err(SpeechError {
          message: "Unknown synthesizer".to_owned(),
        })
      }
      Some(synthesizer) => synthesizer,
    };
    match (
      synthesizer.as_to_audio_data(),
      synthesizer.as_to_audio_output(),
    ) {
      (None, None) => Err(SpeechError {
        message: "Synthesizer does not support playing or returning audio".to_owned(),
      }),
      (Some(synthesizer), None) => {
        let result = synthesizer.speak(voice, language, rate, volume, pitch, text)?;
        let buffer = result
          .pcm
          .chunks_exact(2)
          .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
          .collect::<Vec<i16>>();
        let source = SamplesBuffer::new(1, result.sample_rate, buffer);
        if interrupt {
          SINK.get().unwrap().stop();
        };
        SINK.get().unwrap().append(source);
        Ok(())
      }
      (_, Some(synthesizer)) => {
        synthesizer.speak(voice, language, rate, volume, pitch, text, interrupt)
      }
    }
  })
}
pub fn speak_to_audio_output(
  synthesizer: Option<&str>,
  voice: Option<&str>,
  language: Option<&str>,
  rate: Option<u8>,
  volume: Option<u8>,
  pitch: Option<u8>,
  text: &str,
  interrupt: bool,
) -> Result<(), SpeechError> {
  OPERATION_TX
    .get()
    .unwrap()
    .send(Operation::SpeakToAudioOutput(
      synthesizer.map(|value| value.to_owned()),
      voice.map(|value| value.to_owned()),
      language.map(|value| value.to_owned()),
      rate,
      volume,
      pitch,
      text.to_owned(),
      interrupt,
    ))?;
  match RESULT_RX.lock()?.get().unwrap().recv()? {
    ResultValue::SpeakToAudioOutput(result) => result,
    _ => panic!("Received result value for other operation"),
  }
}
fn internal_stop_speech(synthesizer: &str) -> Result<(), SpeechError> {
  SYNTHESIZERS.with_borrow(|synthesizers| match synthesizers.get(synthesizer) {
    None => Err(SpeechError {
      message: "Unknown synthesizer".to_owned(),
    }),
    Some(synthesizer) => match synthesizer.as_to_audio_output() {
      None => {
        SINK.get().unwrap().stop();
        Ok(())
      }
      Some(synthesizer) => synthesizer.stop_speech(),
    },
  })
}
pub fn stop_speech(synthesizer: &str) -> Result<(), SpeechError> {
  OPERATION_TX
    .get()
    .unwrap()
    .send(Operation::StopSpeech(synthesizer.to_owned()))?;
  match RESULT_RX.lock()?.get().unwrap().recv()? {
    ResultValue::StopSpeech(result) => result,
    _ => panic!("Received result value for other operation"),
  }
}
