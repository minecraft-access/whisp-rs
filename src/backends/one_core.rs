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
      synthesizer: Synthesizer::new().map_err(SpeechError::into_unknown)?,
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
    let voices = Synthesizer::AllVoices()
      .map_err(SpeechError::into_unknown)?
      .into_iter()
      .flat_map(|voice| {
        let display_name = voice.DisplayName()?.to_string();
        let name = voice.Id()?.to_string();
        let languages = vec![voice.Language()?.to_string().to_lowercase()];
        Ok::<Voice, anyhow::Error>(Voice {
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
    voice_name: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> std::result::Result<SpeechResult, SpeechError> {
    let voice = match (voice_name, language) {
      (None, None) => Synthesizer::DefaultVoice()
        .map_err(|err| SpeechError::into_speak_failed(&self.data().name, "default", err))?,
      (Some(voice_name), _) => Synthesizer::AllVoices()
        .map_err(|err| SpeechError::into_speak_failed(&self.data().name, voice_name, err))?
        .into_iter()
        .find(|voice| voice.Id().unwrap() == voice_name)
        .ok_or(SpeechError::into_voice_not_found(voice_name))?,
      (None, Some(language)) => Synthesizer::AllVoices()
        .map_err(|err| SpeechError::into_speak_failed(&self.data().name, language, err))?
        .into_iter()
        .find(|voice| voice.Language().unwrap().to_string().to_lowercase() == language)
        .ok_or(SpeechError::into_language_not_found(language))?,
    };
    self.synthesizer.SetVoice(&voice).map_err(|err| {
      SpeechError::into_speak_failed(
        &self.data().name,
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let options = self.synthesizer.Options().map_err(|err| {
      SpeechError::into_speak_failed(
        &self.data().name,
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let rate = rate.unwrap_or(50) as f64;
    let rate = (rate / 100.0 * 5.5) + 0.5;
    options.SetSpeakingRate(rate).map_err(|err| {
      SpeechError::into_speak_failed(
        &self.data().name,
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let pitch = pitch.unwrap_or(50) as f64;
    let pitch = pitch / 50.0;
    options.SetAudioPitch(pitch).map_err(|err| {
      SpeechError::into_speak_failed(
        &self.data().name,
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let volume = volume.unwrap_or(100) as f64;
    let volume = volume / 100.0;
    options.SetAudioVolume(volume).map_err(|err| {
      SpeechError::into_speak_failed(
        &self.data().name,
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    let text: HSTRING = text.into();
    let result = self
      .synthesizer
      .SynthesizeTextToStreamAsync(&text)
      .map_err(|err| {
        SpeechError::into_speak_failed(
          &self.data().name,
          voice_name.unwrap_or(language.unwrap_or("default")),
          err,
        )
      })?;
    let stream = result.get().map_err(|err| {
      SpeechError::into_speak_failed(
        &self.data().name,
        voice_name.unwrap_or(language.unwrap_or("default")),
        err,
      )
    })?;
    stream.Seek(0).map_err(SpeechError::into_unknown)?;
    let size = stream.Size().map_err(SpeechError::into_unknown)?;
    let buffer = Buffer::Create(size as _).map_err(SpeechError::into_unknown)?;
    stream
      .ReadAsync(&buffer, size as _, InputStreamOptions::None)
      .map_err(SpeechError::into_unknown)?
      .get()
      .map_err(SpeechError::into_unknown)?;
    let memory_buffer =
      Buffer::CreateMemoryBufferOverIBuffer(&buffer).map_err(SpeechError::into_unknown)?;
    let memory_buffer_reference = memory_buffer
      .CreateReference()
      .map_err(SpeechError::into_unknown)?;
    let memory_buffer_accessor: IMemoryBufferByteAccess = memory_buffer_reference
      .cast()
      .map_err(SpeechError::into_unknown)?;
    let mut data_ptr: *mut u8 = std::ptr::null_mut();
    let mut capacity: u32 = 0;
    unsafe {
      memory_buffer_accessor
        .GetBuffer(&mut data_ptr, &mut capacity)
        .map_err(SpeechError::into_unknown)?
    };
    let data = unsafe { std::slice::from_raw_parts_mut(data_ptr, capacity as _) };
    let data_stream = Cursor::new(data);
    let decoder = Decoder::new(data_stream).map_err(SpeechError::into_unknown)?;
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
