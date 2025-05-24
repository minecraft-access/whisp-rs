use crate::audio::*;
#[cfg(target_os = "macos")]
use crate::backends::av_speech_synthesizer::AvSpeechSynthesizer;
use crate::backends::espeak_ng::EspeakNg;
#[cfg(windows)]
use crate::backends::jaws::Jaws;
#[cfg(windows)]
use crate::backends::nvda::Nvda;
#[cfg(windows)]
use crate::backends::one_core::OneCore;
#[cfg(windows)]
use crate::backends::sapi::Sapi;
#[cfg(target_os = "linux")]
use crate::backends::speech_dispatcher::SpeechDispatcher;
use crate::error::SpeechError;
use crate::speech_synthesizer::*;
use anyhow::anyhow;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::any::Any;
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::sync::{mpsc, OnceLock};
use std::thread;
thread_local! {
  static SYNTHESIZERS: RefCell<HashMap<String, Box<dyn SpeechSynthesizer>>> = RefCell::new(HashMap::new());
  static OUTPUT_STREAM: OnceCell<Option<OutputStream>> = const {OnceCell::new() };
}
static SINK: OnceLock<Sink> = OnceLock::new();
type OperationOk = Box<dyn Any + Send + Sync>;
type OperationResult = Result<OperationOk, SpeechError>;
type Operation = Box<dyn FnOnce() -> OperationResult + Send + Sync>;
static OPERATION_TX: OnceLock<mpsc::Sender<(Operation, mpsc::Sender<OperationResult>)>> =
  OnceLock::new();
pub fn initialize() -> Result<(), SpeechError> {
  let (operation_tx, operation_rx) = mpsc::channel();
  OPERATION_TX
    .set(operation_tx)
    .map_err(|_| SpeechError::into_initialize_failed(anyhow!("Failed to set OPERATION_TX")))?;
  let (result_tx, result_rx) = mpsc::channel();
  thread::spawn(move || {
    let closure = || {
      let (output_stream, output_stream_handle) =
        OutputStream::try_default().map_err(SpeechError::into_initialize_failed)?;
      let sink =
        Sink::try_new(&output_stream_handle).map_err(SpeechError::into_initialize_failed)?;
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
    result_tx.send(closure()).unwrap();
    for (operation, sender) in operation_rx {
      sender.send(operation()).unwrap();
    }
  });
  result_rx
    .recv()
    .map_err(SpeechError::into_initialize_failed)?
}
pub fn perform_operation(closure: Operation) -> OperationResult {
  let (result_tx, result_rx) = mpsc::channel();
  OPERATION_TX
    .get()
    .ok_or(SpeechError::into_unknown(anyhow!(
      "OPERATION_TX contains no channel"
    )))?
    .send((closure, result_tx))
    .map_err(SpeechError::into_unknown)?;
  result_rx.recv().map_err(SpeechError::into_unknown)?
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
  let closure = || Ok(Box::new(internal_list_voices()?) as OperationOk);
  let result = perform_operation(Box::new(closure))?
    .downcast()
    .map_err(|_| SpeechError::into_unknown(anyhow!("Failed to downcast received return value")))?;
  Ok(*result)
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
        .ok_or(match (voice, language) {
          (None, None) => SpeechError::NoVoices,
          (Some(voice), _) => SpeechError::into_voice_not_found(voice),
          (None, Some(language)) => SpeechError::into_language_not_found(language),
        })?
        .synthesizer
        .name
        .clone()
    }
  };
  Ok(synthesizer)
}
fn check_parameters(
  rate: Option<u8>,
  volume: Option<u8>,
  pitch: Option<u8>,
) -> Result<(), SpeechError> {
  if rate.is_some_and(|rate| rate > 100) {
    Err(SpeechError::InvalidRate(rate.unwrap()))?;
  };
  if volume.is_some_and(|volume| volume > 100) {
    Err(SpeechError::InvalidVolume(volume.unwrap()))?;
  };
  if pitch.is_some_and(|pitch| pitch > 100) {
    Err(SpeechError::InvalidPitch(pitch.unwrap()))?;
  };
  Ok(())
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
  check_parameters(rate, volume, pitch)?;
  let synthesizer = synthesizer.map(|value| value.to_owned());
  let voice = voice.map(|value| value.to_owned());
  let language = language.map(|value| value.to_owned());
  let text = text.to_owned();
  let closure = move || {
    SYNTHESIZERS.with_borrow(|synthesizers| {
      let synthesizer_name = filter_synthesizers(
        synthesizer.as_deref(),
        voice.as_deref(),
        language.as_deref(),
      )?;
      let synthesizer = synthesizers
        .get(&synthesizer_name)
        .ok_or(SpeechError::into_synthesizer_not_found(&synthesizer_name))?;
      let result = match synthesizer.as_to_audio_data() {
        None => Err(SpeechError::into_audio_data_not_supported(
          &synthesizer_name,
        ))?,
        Some(synthesizer) => synthesizer.speak(
          voice.as_deref(),
          language.as_deref(),
          rate,
          volume,
          pitch,
          &text,
        )?,
      };
      Ok(Box::new(result) as OperationOk)
    })
  };
  let result = perform_operation(Box::new(closure))?
    .downcast()
    .map_err(|_| SpeechError::into_unknown(anyhow!("Failed to downcast received return value")))?;
  Ok(*result)
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
  check_parameters(rate, volume, pitch)?;
  let synthesizer = synthesizer.map(|value| value.to_owned());
  let voice = voice.map(|value| value.to_owned());
  let language = language.map(|value| value.to_owned());
  let text = text.to_owned();
  let closure = move || {
    SYNTHESIZERS.with_borrow(|synthesizers| {
      let synthesizer_name = filter_synthesizers(
        synthesizer.as_deref(),
        voice.as_deref(),
        language.as_deref(),
      )?;
      let synthesizer = synthesizers
        .get(&synthesizer_name)
        .ok_or(SpeechError::into_synthesizer_not_found(&synthesizer_name))?;
      match (
        synthesizer.as_to_audio_data(),
        synthesizer.as_to_audio_output(),
      ) {
        (None, None) => Err(SpeechError::into_speech_not_supported(&synthesizer_name))?,
        (Some(synthesizer), None) => {
          let result = synthesizer.speak(
            voice.as_deref(),
            language.as_deref(),
            rate,
            volume,
            pitch,
            &text,
          )?;
          let buffer = result
            .pcm
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<i16>>();
          let source = SamplesBuffer::new(1, result.sample_rate, buffer);
          if interrupt {
            SINK
              .get()
              .ok_or(SpeechError::into_unknown(anyhow!("SINK contains nothing")))?
              .stop();
          };
          SINK
            .get()
            .ok_or(SpeechError::into_unknown(anyhow!("SINK contains nothing")))?
            .append(source)
        }
        (_, Some(synthesizer)) => synthesizer.speak(
          voice.as_deref(),
          language.as_deref(),
          rate,
          volume,
          pitch,
          &text,
          interrupt,
        )?,
      };
      Ok(Box::new(()) as OperationOk)
    })
  };
  perform_operation(Box::new(closure))?;
  Ok(())
}
pub fn stop_speech(synthesizer: Option<&str>) -> Result<(), SpeechError> {
  let synthesizer = synthesizer.map(|value| value.to_owned());
  let closure = move || {
    SYNTHESIZERS.with_borrow(|synthesizers| {
      match synthesizer {
        Some(synthesizer_name) => {
          let synthesizer = synthesizers
            .get(&synthesizer_name)
            .ok_or(SpeechError::into_synthesizer_not_found(&synthesizer_name))?;
          match (
            synthesizer.as_to_audio_data(),
            synthesizer.as_to_audio_output(),
          ) {
            (None, None) => Err(SpeechError::into_speech_not_supported(&synthesizer_name))?,
            (Some(_), None) => SINK
              .get()
              .ok_or(SpeechError::into_unknown(anyhow!("SINK contains nothing")))?
              .stop(),
            (_, Some(synthesizer)) => synthesizer.stop_speech()?,
          }
        }
        None => {
          SINK
            .get()
            .ok_or(SpeechError::into_unknown(anyhow!("SINK contains nothing")))?
            .stop();
          for synthesizer in synthesizers
            .iter()
            .flat_map(|synthesizer| synthesizer.1.as_to_audio_output())
          {
            let _result = synthesizer.stop_speech();
          }
        }
      };
      Ok(Box::new(()) as OperationOk)
    })
  };
  perform_operation(Box::new(closure))?;
  Ok(())
}
