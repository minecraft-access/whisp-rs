use windows::Media::SpeechSynthesis::*;
use windows::Media::SpeechSynthesis::SpeechSynthesizer as Synthesizer;
use crate::speech_synthesizer::SpeechSynthesizer;
use crate::speech_synthesizer::*;
pub struct OneCore {
  synthesizer: Synthesizer
}
impl SpeechSynthesizer for OneCore {
  fn new() -> Result<Self, SpeechError> {
    Ok(OneCore { synthesizer: Synthesizer::new()? })
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData { name: "OneCore".to_owned(), supports_to_audio_data: false, supports_to_audio_output: false, supports_speech_parameters: false }
  }
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError> {
    let voices = Synthesizer::AllVoices()?
      .into_iter()
      .map(|voice| {
        let display_name = voice.DisplayName().unwrap().to_string();
        let name = voice.Id().unwrap().to_string();
        let languages = vec!(voice.Language().unwrap().to_string().to_lowercase());
        Voice { synthesizer: self.data(), display_name, name, languages, priority: 1 }
      })
      .collect::<Vec<Voice>>();
    Ok(voices)
  }
  fn as_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    None
  }
  fn as_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    None
  }
}
