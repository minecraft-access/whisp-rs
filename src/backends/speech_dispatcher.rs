use crate::error::SpeechError;
use crate::speech_synthesizer::*;
use ssip_client_async::*;
use std::cell::RefCell;
pub struct SpeechDispatcher {
  default_output_module: String,
  default_language: String,
  client: RefCell<Client<fifo::UnixStream>>,
}
impl SpeechSynthesizer for SpeechDispatcher {
  fn new() -> Result<Self, SpeechError> {
    let mut client = fifo::Builder::new()
      .build()
      .map_err(SpeechError::into_unknown)?;
    client
      .set_client_name(ClientName::new("", "audio-navigation-tts"))
      .map_err(SpeechError::into_unknown)?
      .check_client_name_set()
      .map_err(SpeechError::into_unknown)?;
    let default_output_module = client
      .get_output_module()
      .map_err(SpeechError::into_unknown)?
      .receive_string(OK_GET)
      .map_err(SpeechError::into_unknown)?;
    let default_language = client
      .get_language()
      .map_err(SpeechError::into_unknown)?
      .receive_string(OK_GET)
      .map_err(SpeechError::into_unknown)?;
    Ok(SpeechDispatcher {
      default_output_module,
      default_language,
      client: RefCell::new(client),
    })
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData {
      name: "Speech Dispatcher".to_owned(),
      supports_to_audio_data: false,
      supports_to_audio_output: true,
      supports_speech_parameters: true,
    }
  }
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError> {
    let mut client = self.client.borrow_mut();
    let voices = client
      .list_output_modules()
      .map_err(SpeechError::into_unknown)?
      .receive_lines(OK_OUTPUT_MODULES_LIST_SENT)
      .map_err(SpeechError::into_unknown)?
      .into_iter()
      .flat_map(|module| {
        client
          .set_output_module(ClientScope::Current, &module)?
          .check_status(OK_OUTPUT_MODULE_SET)?;
        client
          .list_synthesis_voices()?
          .receive_synthesis_voices()?
          .into_iter()
          .map(|voice| {
            let name = voice.name;
            let languages = match voice.language {
              Some(language) => vec![language.to_lowercase().replace("_", "-")],
              None => Vec::new(),
            };
            let display_name = name.clone() + " (" + &module + ")";
            let name = module.clone() + "/" + &name;
            Ok(Voice {
              synthesizer: self.data(),
              display_name,
              name,
              languages,
              priority: 1,
            })
          })
          .collect::<Result<Vec<Voice>, anyhow::Error>>()
      })
      .flatten()
      .collect();
    Ok(voices)
  }
  fn as_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    None
  }
  fn as_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    Some(self)
  }
}
impl SpeechSynthesizerToAudioOutput for SpeechDispatcher {
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
    interrupt: bool,
  ) -> std::result::Result<(), SpeechError> {
    let voice = match (voice, language) {
      (None, None) => None,
      (Some(voice), _) => Some(voice.to_owned()),
      (_, Some(language)) => Some(
        self
          .list_voices()?
          .into_iter()
          .find(|voice| voice.languages.iter().any(|name| name == language))
          .ok_or(SpeechError::into_language_not_found(language))?
          .name,
      ),
    };
    let mut client = self.client.borrow_mut();
    match voice {
      None => {
        client
          .set_output_module(ClientScope::Current, &self.default_output_module)
          .map_err(|err| {
            SpeechError::into_speak_failed(&self.data().name, &self.default_output_module, err)
          })?
          .check_status(OK_OUTPUT_MODULE_SET)
          .map_err(|err| {
            SpeechError::into_speak_failed(&self.data().name, &self.default_output_module, err)
          })?;
        client
          .set_language(ClientScope::Current, &self.default_language)
          .map_err(|err| {
            SpeechError::into_speak_failed(&self.data().name, &self.default_language, err)
          })?
          .check_status(OK_LANGUAGE_SET)
          .map_err(|err| {
            SpeechError::into_speak_failed(&self.data().name, &self.default_language, err)
          })?;
      }
      Some(ref voice) => {
        let mut split = voice.split('/');
        let output_module = split.next().unwrap();
        let voice = split.next().unwrap();
        client
          .set_output_module(ClientScope::Current, output_module)
          .map_err(|err| SpeechError::into_speak_failed(&self.data().name, output_module, err))?
          .check_status(OK_OUTPUT_MODULE_SET)
          .map_err(|err| SpeechError::into_speak_failed(&self.data().name, output_module, err))?;
        client
          .set_synthesis_voice(ClientScope::Current, voice)
          .map_err(|err| SpeechError::into_speak_failed(&self.data().name, voice, err))?
          .check_status(OK_VOICE_SET)
          .map_err(|err| SpeechError::into_speak_failed(&self.data().name, voice, err))?;
      }
    };
    let rate = rate.unwrap_or(50) as i8;
    let rate = (rate * 2) - 100;
    client
      .set_rate(ClientScope::Current, rate)
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?
      .check_status(OK_RATE_SET)
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?;
    let pitch = pitch.unwrap_or(50) as i8;
    let pitch = (pitch * 2) - 100;
    client
      .set_pitch(ClientScope::Current, pitch)
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?
      .check_status(OK_PITCH_SET)
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?;
    let volume = volume.unwrap_or(50) as i8;
    let volume = (volume * 2) - 100;
    client
      .set_volume(ClientScope::Current, volume)
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?
      .check_status(OK_VOLUME_SET)
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?;
    if interrupt {
      client
        .cancel(MessageScope::Last)
        .map_err(|err| SpeechError::into_stop_speech_failed(&self.data().name, err))?
        .check_status(OK_CANCELED)
        .map_err(|err| SpeechError::into_stop_speech_failed(&self.data().name, err))?;
    };
    let lines = text
      .lines()
      .map(|line| line.to_owned())
      .collect::<Vec<String>>();
    client
      .speak()
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?
      .check_receiving_data()
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?
      .send_lines(&lines)
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?
      .receive_message_id()
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice.as_deref().unwrap_or(&self.default_output_module),
          err,
        )
      })?;
    Ok(())
  }
  fn stop_speech(&self) -> std::result::Result<(), SpeechError> {
    self
      .client
      .borrow_mut()
      .cancel(MessageScope::Last)
      .map_err(|err| SpeechError::into_stop_speech_failed(&self.data().name, err))?
      .check_status(OK_CANCELED)
      .map_err(|err| SpeechError::into_stop_speech_failed(&self.data().name, err))?;
    Ok(())
  }
}
