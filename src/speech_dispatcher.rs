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
    let mut client = fifo::Builder::new().build()?;
    client
      .set_client_name(ClientName::new("", "audio-navigation-tts"))?
      .check_client_name_set()?;
    let default_output_module = client.get_output_module()?.receive_string(OK_GET)?;
    let default_language = client.get_language()?.receive_string(OK_GET)?;
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
      .list_output_modules()?
      .receive_lines(OK_OUTPUT_MODULES_LIST_SENT)?
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
          .collect::<Result<Vec<Voice>, SpeechError>>()
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
    let mut client = self.client.borrow_mut();
    let voice = match (voice, language) {
      (None, None) => None,
      (Some(voice), _) => Some(voice.to_owned()),
      (_, Some(language)) => Some(
        self
          .list_voices()?
          .into_iter()
          .find(|voice| voice.languages.iter().any(|name| name == language))
          .ok_or(SpeechError {
            message: "Voice not found".to_owned(),
          })?
          .name,
      ),
    };
    match voice {
      None => {
        client
          .set_output_module(ClientScope::Current, &self.default_output_module)?
          .check_status(OK_OUTPUT_MODULE_SET)?;
        client
          .set_language(ClientScope::Current, &self.default_language)?
          .check_status(OK_LANGUAGE_SET)?;
      }
      Some(voice) => {
        let mut split = voice.split('/');
        let output_module = split.next().unwrap();
        let voice = split.next().unwrap();
        client
          .set_output_module(ClientScope::Current, output_module)?
          .check_status(OK_OUTPUT_MODULE_SET)?;
        client
          .set_synthesis_voice(ClientScope::Current, voice)?
          .check_status(OK_VOICE_SET)?;
      }
    };
    let rate = rate.unwrap_or(50) as i8;
    let rate = (rate * 2) - 100;
    client
      .set_rate(ClientScope::Current, rate)?
      .check_status(OK_RATE_SET)?;
    let pitch = pitch.unwrap_or(50) as i8;
    let pitch = (pitch * 2) - 100;
    client
      .set_pitch(ClientScope::Current, pitch)?
      .check_status(OK_PITCH_SET)?;
    let volume = volume.unwrap_or(50) as i8;
    let volume = (volume * 2) - 100;
    client
      .set_volume(ClientScope::Current, volume)?
      .check_status(OK_VOLUME_SET)?;
    if interrupt {
      client
        .cancel(MessageScope::Last)?
        .check_status(OK_CANCELED)?;
    };
    let lines = text
      .lines()
      .map(|line| line.to_owned())
      .collect::<Vec<String>>();
    client
      .speak()?
      .check_receiving_data()?
      .send_lines(&lines)?
      .receive_message_id()?;
    Ok(())
  }
  fn stop_speech(&self) -> std::result::Result<(), SpeechError> {
    self
      .client
      .borrow_mut()
      .cancel(MessageScope::Last)?
      .check_status(OK_CANCELED)?;
    Ok(())
  }
}
