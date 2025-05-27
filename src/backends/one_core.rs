use crate::audio::{SampleFormat, SpeechResult};
use crate::backends::{
  Backend, BrailleBackend, SpeechSynthesizerToAudioData, SpeechSynthesizerToAudioOutput,
};
use crate::error::OutputError;
use crate::metadata::Voice;
use rodio::{Decoder, Source};
use std::io::Cursor;
use windows::core::{Interface, HSTRING};
use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::{Buffer, InputStreamOptions};
use windows::Win32::System::WinRT::IMemoryBufferByteAccess;
pub struct OneCore {
  synthesizer: SpeechSynthesizer,
}
impl Backend for OneCore {
  fn new() -> std::result::Result<Self, OutputError> {
    Ok(OneCore {
      synthesizer: SpeechSynthesizer::new().map_err(OutputError::into_unknown)?,
    })
  }
  fn name(&self) -> String {
    "OneCore".to_owned()
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, OutputError> {
    let voices = SpeechSynthesizer::AllVoices()
      .map_err(OutputError::into_unknown)?
      .into_iter()
      .flat_map(|voice| {
        let display_name = voice.DisplayName()?.to_string();
        let name = voice.Id()?.to_string();
        let languages = vec![voice.Language()?.to_string().to_lowercase()];
        Ok::<Voice, anyhow::Error>(Voice {
          synthesizer: self.speech_metadata().unwrap(),
          display_name,
          name,
          languages,
          priority: 1,
        })
      })
      .collect::<Vec<Voice>>();
    Ok(voices)
  }
  fn as_speech_synthesizer_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    Some(self)
  }
  fn as_speech_synthesizer_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    None
  }
  fn as_braille_backend(&self) -> Option<&dyn BrailleBackend> {
    None
  }
}
impl SpeechSynthesizerToAudioData for OneCore {
  fn supports_speech_parameters(&self) -> bool {
    true
  }
  #[allow(clippy::too_many_lines)]
  fn speak(
    &self,
    voice_name: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> std::result::Result<SpeechResult, OutputError> {
    let voice = match (voice_name, language) {
      (None, None) => SpeechSynthesizer::DefaultVoice()
        .map_err(|err| OutputError::into_speak_failed(&self.name(), "default", err))?,
      (Some(voice_name), _) => SpeechSynthesizer::AllVoices()
        .map_err(|err| OutputError::into_speak_failed(&self.name(), voice_name, err))?
        .into_iter()
        .find(|voice| voice.Id().unwrap() == voice_name)
        .ok_or(OutputError::into_voice_not_found(voice_name))?,
      (None, Some(language)) => SpeechSynthesizer::AllVoices()
        .map_err(|err| OutputError::into_speak_failed(&self.name(), language, err))?
        .into_iter()
        .find(|voice| voice.Language().unwrap().to_string().to_lowercase() == language)
        .ok_or(OutputError::into_language_not_found(language))?,
    };
    self.synthesizer.SetVoice(&voice).map_err(|err| {
      OutputError::into_speak_failed(
        &self.name(),
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let options = self.synthesizer.Options().map_err(|err| {
      OutputError::into_speak_failed(
        &self.name(),
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let rate = f64::from(rate.unwrap_or(50));
    let rate = (rate / 100.0 * 5.5) + 0.5;
    options.SetSpeakingRate(rate).map_err(|err| {
      OutputError::into_speak_failed(
        &self.name(),
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let pitch = f64::from(pitch.unwrap_or(50));
    let pitch = pitch / 50.0;
    options.SetAudioPitch(pitch).map_err(|err| {
      OutputError::into_speak_failed(
        &self.name(),
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let volume = f64::from(volume.unwrap_or(100));
    let volume = volume / 100.0;
    options.SetAudioVolume(volume).map_err(|err| {
      OutputError::into_speak_failed(
        &self.name(),
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let text: HSTRING = text.into();
    let result = self
      .synthesizer
      .SynthesizeTextToStreamAsync(&text)
      .map_err(|err| {
        OutputError::into_speak_failed(
          &self.name(),
          voice_name.unwrap_or(language.unwrap_or("default")),
          err,
        )
      })?;
    let stream = result.get().map_err(|err| {
      OutputError::into_speak_failed(
        &self.name(),
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    stream.Seek(0).map_err(OutputError::into_unknown)?;
    let size = stream.Size().map_err(OutputError::into_unknown)?;
    let buffer = Buffer::Create(size.try_into().map_err(OutputError::into_unknown)?)
      .map_err(OutputError::into_unknown)?;
    stream
      .ReadAsync(
        &buffer,
        size.try_into().map_err(OutputError::into_unknown)?,
        InputStreamOptions::None,
      )
      .map_err(OutputError::into_unknown)?
      .get()
      .map_err(OutputError::into_unknown)?;
    let memory_buffer =
      Buffer::CreateMemoryBufferOverIBuffer(&buffer).map_err(OutputError::into_unknown)?;
    let memory_buffer_reference = memory_buffer
      .CreateReference()
      .map_err(OutputError::into_unknown)?;
    let memory_buffer_accessor: IMemoryBufferByteAccess = memory_buffer_reference
      .cast()
      .map_err(OutputError::into_unknown)?;
    let mut data_ptr: *mut u8 = std::ptr::null_mut();
    let mut capacity: u32 = 0;
    unsafe {
      memory_buffer_accessor
        .GetBuffer(&mut data_ptr, &mut capacity)
        .map_err(OutputError::into_unknown)?;
    };
    let data = unsafe {
      std::slice::from_raw_parts_mut(
        data_ptr,
        capacity.try_into().map_err(OutputError::into_unknown)?,
      )
    };
    let data_stream = Cursor::new(data);
    let decoder = Decoder::new(data_stream).map_err(OutputError::into_unknown)?;
    let sample_rate = decoder.sample_rate();
    let pcm = decoder.flat_map(i16::to_le_bytes).collect::<Vec<u8>>();
    Ok(SpeechResult {
      pcm,
      sample_format: SampleFormat::S16,
      sample_rate,
    })
  }
}
