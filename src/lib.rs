#![deny(clippy::all)]
#![warn(clippy::pedantic)]
pub mod audio;
mod backends;
//mod c_api;
pub mod error;
//mod jni;
pub mod metadata;
use crate::audio::SpeechResult;
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
use crate::backends::{Backend, BrailleBackend};
use crate::error::OutputError;
use crate::metadata::{BrailleBackendMetadata, SpeechSynthesizerMetadata, Voice};
use anyhow::anyhow;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::any::Any;
use std::cell::{Cell, OnceCell, RefCell};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
thread_local! {
  static BACKENDS: RefCell<HashMap<String, Box<dyn Backend>>> = RefCell::new(HashMap::new());
  static OUTPUT_STREAM: OnceCell<OutputStream> = const {OnceCell::new() };
  static SINK: OnceCell<Sink> = const { OnceCell::new() };
}
fn stop_audio() -> Result<(), OutputError> {
  SINK.with(|cell| {
    cell
      .get()
      .ok_or(OutputError::into_unknown(anyhow!("SINK contains nothing")))?
      .stop();
    Ok(())
  })
}
fn play_audio(result: &SpeechResult) -> Result<(), OutputError> {
  let buffer = result
    .pcm
    .chunks_exact(2)
    .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
    .collect::<Vec<i16>>();
  let source = SamplesBuffer::new(1, result.sample_rate, buffer);
  SINK.with(|cell| {
    cell
      .get()
      .ok_or(OutputError::into_unknown(anyhow!("SINK contains nothing")))?
      .append(source);
    Ok(())
  })
}
type OperationOk = Box<dyn Any + Send + Sync>;
type OperationResult = Result<OperationOk, OutputError>;
type Operation = Box<dyn FnOnce() -> OperationResult + Send + Sync>;
pub struct Whisprs {
  operation_tx: Mutex<mpsc::Sender<(Operation, mpsc::Sender<OperationResult>)>>,
  should_stop: Arc<AtomicBool>,
  thread_handle: Mutex<Cell<Option<thread::JoinHandle<()>>>>,
}
impl Whisprs {
  pub fn new() -> Result<Self, OutputError> {
    let (operation_tx, operation_rx) =
      mpsc::channel::<(Operation, mpsc::Sender<OperationResult>)>();
    let should_stop = Arc::new(AtomicBool::new(false));
    let (result_tx, result_rx) = mpsc::channel::<Result<(), OutputError>>();
    let thread_should_stop = should_stop.clone();
    let thread_handle = thread::spawn(move || {
      let closure = || {
        let (output_stream, output_stream_handle) =
          OutputStream::try_default().map_err(OutputError::into_initialize_failed)?;
        let sink =
          Sink::try_new(&output_stream_handle).map_err(OutputError::into_initialize_failed)?;
        let _result = OUTPUT_STREAM.with(|cell| cell.set(output_stream));
        let _result = SINK.with(|cell| cell.set(sink));
        let mut backends: Vec<Result<Box<dyn Backend>, OutputError>> = Vec::new();
        backends.push(EspeakNg::new().map(|value| Box::new(value) as Box<dyn Backend>));
        #[cfg(windows)]
        {
          backends.push(Sapi::new().map(|value| Box::new(value) as Box<dyn Backend>));
          backends.push(OneCore::new().map(|value| Box::new(value) as Box<dyn Backend>));
          backends.push(Jaws::new().map(|value| Box::new(value) as Box<dyn Backend>));
          backends.push(Nvda::new().map(|value| Box::new(value) as Box<dyn Backend>));
        }
        #[cfg(target_os = "linux")]
        {
          backends.push(SpeechDispatcher::new().map(|value| Box::new(value) as Box<dyn Backend>));
        }
        #[cfg(target_os = "macos")]
        {
          backends
            .push(AvSpeechSynthesizer::new().map(|value| Box::new(value) as Box<dyn Backend>));
        }
        BACKENDS.set(
          backends
            .into_iter()
            .flatten()
            .map(|backend| (backend.name(), backend))
            .collect(),
        );
        Ok(())
      };
      result_tx.send(closure()).unwrap();
      for (operation, sender) in operation_rx {
        sender.send(operation()).unwrap();
        if thread_should_stop.load(Ordering::Relaxed) {
          return;
        }
      }
    });
    result_rx
      .recv()
      .map_err(OutputError::into_initialize_failed)??;
    Ok(Whisprs {
      operation_tx: Mutex::new(operation_tx),
      should_stop,
      thread_handle: Mutex::new(Cell::new(Some(thread_handle))),
    })
  }
  fn perform_operation(&self, closure: Operation) -> OperationResult {
    let (result_tx, result_rx) = mpsc::channel();
    self
      .operation_tx
      .lock()
      .map_err(|_| OutputError::into_unknown(anyhow!("Failed to lock operation_tx")))?
      .send((closure, result_tx))
      .map_err(OutputError::into_unknown)?;
    result_rx.recv().map_err(OutputError::into_unknown)?
  }
  fn internal_list_voices(
    synthesizer: Option<&str>,
    name: Option<&str>,
    language: Option<&str>,
    needs_audio_data: bool,
  ) -> Result<Vec<Voice>, OutputError> {
    BACKENDS.with_borrow(|backends| {
      let mut voices = backends
        .values()
        .filter(|backend| synthesizer.is_none_or(|synthesizer| backend.name() == synthesizer))
        .filter(|synthesizer| {
          !needs_audio_data || synthesizer.as_speech_synthesizer_to_audio_data().is_some()
        })
        .flat_map(|backend| backend.list_voices())
        .flatten()
        .filter(|voice| {
          name.is_none_or(|name| voice.name == name)
            && language.is_none_or(|name| {
              voice.languages.is_empty() || voice.languages.iter().any(|language| language == name)
            })
        })
        .collect::<Vec<Voice>>();
      voices.sort_unstable_by_key(|voice| (voice.priority, voice.name.clone()));
      Ok(voices)
    })
  }
  pub fn list_voices(
    &self,
    synthesizer: Option<&str>,
    name: Option<&str>,
    language: Option<&str>,
    needs_audio_data: bool,
  ) -> Result<Vec<Voice>, OutputError> {
    let synthesizer = synthesizer.map(std::borrow::ToOwned::to_owned);
    let name = name.map(std::borrow::ToOwned::to_owned);
    let language = language.map(std::borrow::ToOwned::to_owned);
    let closure = move || {
      Ok(Box::new(Whisprs::internal_list_voices(
        synthesizer.as_deref(),
        name.as_deref(),
        language.as_deref(),
        needs_audio_data,
      )?) as OperationOk)
    };
    let result = self
      .perform_operation(Box::new(closure))?
      .downcast()
      .map_err(|_| {
        OutputError::into_unknown(anyhow!("Failed to downcast received return value"))
      })?;
    Ok(*result)
  }
  pub fn list_speech_synthesizers(&self) -> Result<Vec<SpeechSynthesizerMetadata>, OutputError> {
    let closure = || {
      BACKENDS.with_borrow(|backends| {
        let synthesizers = backends
          .values()
          .filter_map(|backend| backend.speech_metadata())
          .collect::<Vec<SpeechSynthesizerMetadata>>();
        Ok(Box::new(synthesizers) as OperationOk)
      })
    };
    let result = self
      .perform_operation(Box::new(closure))?
      .downcast()
      .map_err(|_| {
        OutputError::into_unknown(anyhow!("Failed to downcast received return value"))
      })?;
    Ok(*result)
  }
  pub fn list_speech_synthesizers_supporting_audio_data(
    &self,
  ) -> Result<Vec<SpeechSynthesizerMetadata>, OutputError> {
    let closure = || {
      BACKENDS.with_borrow(|backends| {
        let synthesizers = backends
          .values()
          .filter_map(|backend| backend.speech_metadata())
          .filter(|synthesizer| synthesizer.supports_speaking_to_audio_data)
          .collect::<Vec<SpeechSynthesizerMetadata>>();
        Ok(Box::new(synthesizers) as OperationOk)
      })
    };
    let result = self
      .perform_operation(Box::new(closure))?
      .downcast()
      .map_err(|_| {
        OutputError::into_unknown(anyhow!("Failed to downcast received return value"))
      })?;
    Ok(*result)
  }
  pub fn list_braille_backends(&self) -> Result<Vec<BrailleBackendMetadata>, OutputError> {
    let closure = || {
      BACKENDS.with_borrow(|backends| {
        let backends = backends
          .values()
          .filter_map(|backend| backend.braille_metadata())
          .collect::<Vec<BrailleBackendMetadata>>();
        Ok(Box::new(backends) as OperationOk)
      })
    };
    let result = self
      .perform_operation(Box::new(closure))?
      .downcast()
      .map_err(|_| {
        OutputError::into_unknown(anyhow!("Failed to downcast received return value"))
      })?;
    Ok(*result)
  }
  fn filter_synthesizers(
    synthesizer: Option<&str>,
    voice: Option<&str>,
    language: Option<&str>,
    needs_audio_data: bool,
  ) -> Result<String, OutputError> {
    let synthesizer = match (synthesizer, voice, language) {
      (Some(synthesizer), _, _) => synthesizer.to_owned(),
      (None, voice_name, language) => {
        let voices = Whisprs::internal_list_voices(None, voice_name, language, needs_audio_data)?;
        voices
          .first()
          .ok_or(match (voice, language) {
            (None, None) => OutputError::NoVoices,
            (Some(voice), _) => OutputError::into_voice_not_found(voice),
            (None, Some(language)) => OutputError::into_language_not_found(language),
          })?
          .synthesizer
          .name
          .clone()
      }
    };
    Ok(synthesizer)
  }
  fn check_speech_parameters(
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
  ) -> Result<(), OutputError> {
    if rate.is_some_and(|rate| rate > 100) {
      Err(OutputError::InvalidRate(rate.unwrap()))?;
    }
    if volume.is_some_and(|volume| volume > 100) {
      Err(OutputError::InvalidVolume(volume.unwrap()))?;
    }
    if pitch.is_some_and(|pitch| pitch > 100) {
      Err(OutputError::InvalidPitch(pitch.unwrap()))?;
    }
    Ok(())
  }
  pub fn speak_to_audio_data(
    &self,
    synthesizer: Option<&str>,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> Result<SpeechResult, OutputError> {
    Whisprs::check_speech_parameters(rate, volume, pitch)?;
    let synthesizer = synthesizer.map(std::borrow::ToOwned::to_owned);
    let voice = voice.map(std::borrow::ToOwned::to_owned);
    let language = language.map(std::borrow::ToOwned::to_owned);
    let text = text.to_owned();
    let closure = move || {
      BACKENDS.with_borrow(|backends| {
        let synthesizer_name = Whisprs::filter_synthesizers(
          synthesizer.as_deref(),
          voice.as_deref(),
          language.as_deref(),
          true,
        )?;
        let synthesizer = backends
          .get(&synthesizer_name)
          .ok_or(OutputError::into_backend_not_found(&synthesizer_name))?;
        let result = match synthesizer.as_speech_synthesizer_to_audio_data() {
          None => Err(OutputError::into_audio_data_not_supported(
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
    let result = self
      .perform_operation(Box::new(closure))?
      .downcast()
      .map_err(|_| {
        OutputError::into_unknown(anyhow!("Failed to downcast received return value"))
      })?;
    Ok(*result)
  }
  pub fn speak_to_audio_output(
    &self,
    synthesizer: Option<&str>,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
    interrupt: bool,
  ) -> Result<(), OutputError> {
    Whisprs::check_speech_parameters(rate, volume, pitch)?;
    let synthesizer = synthesizer.map(std::borrow::ToOwned::to_owned);
    let voice = voice.map(std::borrow::ToOwned::to_owned);
    let language = language.map(std::borrow::ToOwned::to_owned);
    let text = text.to_owned();
    let closure = move || {
      BACKENDS.with_borrow(|backends| {
        let synthesizer_name = Whisprs::filter_synthesizers(
          synthesizer.as_deref(),
          voice.as_deref(),
          language.as_deref(),
          false,
        )?;
        let synthesizer = backends
          .get(&synthesizer_name)
          .ok_or(OutputError::into_backend_not_found(&synthesizer_name))?;
        match (
          synthesizer.as_speech_synthesizer_to_audio_data(),
          synthesizer.as_speech_synthesizer_to_audio_output(),
        ) {
          (None, None) => Err(OutputError::into_speech_not_supported(&synthesizer_name))?,
          (Some(synthesizer), None) => {
            let result = synthesizer.speak(
              voice.as_deref(),
              language.as_deref(),
              rate,
              volume,
              pitch,
              &text,
            )?;
            if interrupt {
              stop_audio()?;
            }
            play_audio(&result)?;
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
        }
        Ok(Box::new(()) as OperationOk)
      })
    };
    self.perform_operation(Box::new(closure))?;
    Ok(())
  }
  pub fn stop_speech(&self, synthesizer: Option<&str>) -> Result<(), OutputError> {
    let synthesizer = synthesizer.map(std::borrow::ToOwned::to_owned);
    let closure = move || {
      BACKENDS.with_borrow(|backends| {
        if let Some(synthesizer_name) = synthesizer {
          let synthesizer = backends
            .get(&synthesizer_name)
            .ok_or(OutputError::into_backend_not_found(&synthesizer_name))?;
          match (
            synthesizer.as_speech_synthesizer_to_audio_data(),
            synthesizer.as_speech_synthesizer_to_audio_output(),
          ) {
            (None, None) => Err(OutputError::into_speech_not_supported(&synthesizer_name))?,
            (Some(_), None) => stop_audio()?,
            (_, Some(synthesizer)) => synthesizer.stop_speech()?,
          }
        } else {
          stop_audio()?;
          for synthesizer in backends
            .iter()
            .filter_map(|backend| backend.1.as_speech_synthesizer_to_audio_output())
          {
            let _result = synthesizer.stop_speech();
          }
        }
        Ok(Box::new(()) as OperationOk)
      })
    };
    self.perform_operation(Box::new(closure))?;
    Ok(())
  }
  pub fn braille(&self, backend: Option<&str>, text: &str) -> Result<(), OutputError> {
    let backend = backend.map(std::borrow::ToOwned::to_owned);
    let text = text.to_owned();
    let closure = move || {
      BACKENDS.with_borrow(|backends| {
        if let Some(backend_name) = backend {
          backends
            .get(&backend_name)
            .ok_or(OutputError::into_backend_not_found(&backend_name))?
            .as_braille_backend()
            .ok_or(OutputError::into_braille_not_supported(&backend_name))?
            .braille(&text)?;
        } else {
          let mut braille_backends = backends
            .iter()
            .filter_map(|backend| backend.1.as_braille_backend())
            .collect::<Vec<&dyn BrailleBackend>>();
          braille_backends.sort_unstable_by_key(|backend| backend.priority());
          braille_backends
            .first()
            .ok_or(OutputError::NoBrailleBackends)?
            .braille(&text)?;
        }
        Ok(Box::new(()) as OperationOk)
      })
    };
    self.perform_operation(Box::new(closure))?;
    Ok(())
  }
  pub fn output(
    &self,
    synthesizer: Option<&str>,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    braille_backend: Option<&str>,
    text: &str,
    interrupt: bool,
  ) -> Result<(), OutputError> {
    let speech_result = self.speak_to_audio_output(
      synthesizer,
      voice,
      language,
      rate,
      volume,
      pitch,
      text,
      interrupt,
    );
    let braille_result = self.braille(braille_backend, text);
    match (speech_result, braille_result) {
      (Err(OutputError::NoVoices), Err(OutputError::NoBrailleBackends)) => {
        Err(OutputError::NoBackends)
      }
      (Err(OutputError::NoVoices) | Ok(()), right) => right,
      (left, _) => left,
    }
  }
}
impl Drop for Whisprs {
  fn drop(&mut self) {
    self.should_stop.store(true, Ordering::Relaxed);
    let closure = || Ok(Box::new(()) as OperationOk);
    self.perform_operation(Box::new(closure)).unwrap();
    self
      .thread_handle
      .lock()
      .unwrap()
      .take()
      .unwrap()
      .join()
      .unwrap();
  }
}
