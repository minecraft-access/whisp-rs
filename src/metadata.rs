#[derive(Debug)]
pub struct SpeechSynthesizerMetadata {
  pub name: String,
  pub supports_speaking_to_audio_data: bool,
  pub supports_speech_parameters: bool,
}
#[derive(Debug)]
pub struct BrailleBackendMetadata {
  pub name: String,
  pub priority: u8,
}
#[derive(Debug)]
pub struct Voice {
  pub synthesizer: SpeechSynthesizerMetadata,
  pub display_name: String,
  pub name: String,
  pub languages: Vec<String>,
  pub priority: u8,
}
