use crate::speech_synthesizer::*;
use rodio::*;
use std::io::Cursor;
use windows::core::*;
use windows::Media::SpeechSynthesis::SpeechSynthesizer as Synthesizer;
use windows::Storage::Streams::*;
use windows::Win32::System::WinRT::IMemoryBufferByteAccess;
pub struct OneCore {
  synthesizer: Synthesizer,
}
impl SpeechSynthesizer for OneCore {
  fn new() -> std::result::Result<Self, SpeechError> {
    Ok(OneCore {
      synthesizer: Synthesizer::new()?,
    })
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData {
      name: "OneCore".to_owned(),
      supports_to_audio_data: true,
      supports_to_audio_output: false,
      supports_speech_parameters: true,
    }
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, SpeechError> {
    let voices = Synthesizer::AllVoices()?
      .into_iter()
      .flat_map(|voice| {
        let display_name = voice.DisplayName()?.to_string();
        let name = voice.Id()?.to_string();
        let languages = vec![voice.Language()?.to_string().to_lowercase()];
        Ok::<Voice, SpeechError>(Voice {
          synthesizer: self.data(),
          display_name,
          name,
          languages,
          priority: 1,
        })
      })
      .collect::<Vec<Voice>>();
    Ok(voices)
  }
  fn as_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    Some(self)
  }
  fn as_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    None
  }
}
impl SpeechSynthesizerToAudioData for OneCore {
  fn speak(
    &self,
    voice_name: &str,
    _language: &str,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> std::result::Result<SpeechResult, SpeechError> {
    let voice = Synthesizer::AllVoices()?
      .into_iter()
      .find(|voice| voice.Id().unwrap() == voice_name)
      .ok_or(SpeechError {
        message: "Voice not found".to_owned(),
      })?;
    self.synthesizer.SetVoice(&voice)?;
    let options = self.synthesizer.Options()?;
    let rate = rate.unwrap_or(50) as f64;
    let rate = (rate / 100.0 * 5.5) + 0.5;
    options.SetSpeakingRate(rate)?;
    let pitch = pitch.unwrap_or(50) as f64;
    let pitch = pitch / 50.0;
    options.SetAudioPitch(pitch)?;
    let volume = volume.unwrap_or(100) as f64;
    let volume = volume / 100.0;
    options.SetAudioVolume(volume)?;
    let text: HSTRING = text.into();
    let result = self.synthesizer.SynthesizeTextToStreamAsync(&text)?;
    let stream = result.get()?;
    stream.Seek(0)?;
    let size = stream.Size()?;
    let buffer = Buffer::Create(size as _)?;
    stream
      .ReadAsync(&buffer, size as _, InputStreamOptions::None)?
      .get()?;
    let memory_buffer = Buffer::CreateMemoryBufferOverIBuffer(&buffer)?;
    let memory_buffer_reference = memory_buffer.CreateReference()?;
    let memory_buffer_accessor: IMemoryBufferByteAccess = memory_buffer_reference.cast()?;
    let mut data_ptr: *mut u8 = std::ptr::null_mut();
    let mut capacity: u32 = 0;
    unsafe { memory_buffer_accessor.GetBuffer(&mut data_ptr, &mut capacity)? };
    let data = unsafe { std::slice::from_raw_parts_mut(data_ptr, capacity as _) };
    let data_stream = Cursor::new(data);
    let decoder = Decoder::new(data_stream)?;
    let sample_rate = decoder.sample_rate();
    let pcm = decoder
      .flat_map(|sample| sample.to_le_bytes())
      .collect::<Vec<u8>>();
    Ok(SpeechResult {
      pcm,
      sample_format: SampleFormat::S16,
      sample_rate,
    })
  }
}
