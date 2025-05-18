use std::cell::RefCell;
use ssip_client_async::*;
use crate::speech_synthesizer::*;
pub struct SpeechDispatcher {
  client: RefCell<Client<fifo::UnixStream>>
}
impl SpeechSynthesizer for SpeechDispatcher {
  fn new() -> Result<Self, SpeechError> {
    let mut client = fifo::Builder::new()
      .build()?;
    client
      .set_client_name(ClientName::new("", "audio-navigation-tts"))?
      .check_client_name_set()?;
    Ok(SpeechDispatcher { client: RefCell::new(client) })
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData { name: "Speech Dispatcher".to_owned(), supports_to_audio_data: false, supports_to_audio_output: true, supports_speech_parameters: true }
  }
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError> {
    let mut client = self.client.borrow_mut();
    let voices = client
      .list_output_modules()?
      .receive_lines(OK_OUTPUT_MODULES_LIST_SENT)?
      .into_iter()
      .map(|module| {
        client.set_output_module(ClientScope::Current, &module)?.check_status(OK_OUTPUT_MODULE_SET)?;
        client
          .list_synthesis_voices()?
          .receive_synthesis_voices()?
          .into_iter()
          .map(|voice| {
            let name = voice.name;
            let languages = match voice.language {
              Some(language) => vec!(language.to_lowercase()),
              None => Vec::new()
            };
            let display_name = name.clone()+" ("+&module+")";
            let name = module.clone()+"/"+&name;
            Ok(Voice { synthesizer: self.data(), display_name, name, languages, priority: 1 })
          })
          .collect::<Result<Vec<Voice>, SpeechError>>()
      })
      .flatten()
      .flatten()
      .collect();
    Ok(voices)
  }
  fn as_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    None
  }
  fn as_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    None
  }
}
