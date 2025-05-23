use crate::error::SpeechError;
use crate::speech_synthesizer::*;
use anyhow::anyhow;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use std::collections::HashSet;
use std::ffi::c_void;
use std::io::Cursor;
use windows::core::*;
use windows::Win32::Globalization::LCIDToLocaleName;
use windows::Win32::Media::Audio::*;
use windows::Win32::Media::Speech::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::SystemServices::LOCALE_NAME_MAX_LENGTH;
use windows::Win32::UI::Shell::SHCreateMemStream;
fn set_parameters(
  synthesizer: &ISpVoice,
  synthesizer_name: &str,
  default_voice: &str,
  voice: Option<&str>,
  language: Option<&str>,
  rate: Option<u8>,
  volume: Option<u8>,
  pitch: Option<u8>,
  text: &str,
) -> std::result::Result<String, SpeechError> {
  unsafe {
    let voice_token = match (voice, language) {
      (None, None) => {
        let token: ISpObjectToken = CoCreateInstance(&SpObjectToken, None, CLSCTX_ALL)
          .map_err(|err| SpeechError::into_speak_failed(synthesizer_name, default_voice, err))?;
        let mut default_voice_vector = default_voice
          .encode_utf16()
          .chain(Some(0))
          .collect::<Vec<u16>>();
        token
          .SetId(
            SPCAT_VOICES,
            PWSTR::from_raw(default_voice_vector.as_mut_ptr()),
            false,
          )
          .map_err(|err| SpeechError::into_speak_failed(synthesizer_name, default_voice, err))?;
        token
      }
      (None, Some(language)) => {
        let category: ISpObjectTokenCategory =
          CoCreateInstance(&SpObjectTokenCategory, None, CLSCTX_ALL)
            .map_err(|err| SpeechError::into_speak_failed(synthesizer_name, language, err))?;
        category
          .SetId(SPCAT_VOICES, false)
          .map_err(|err| SpeechError::into_speak_failed(synthesizer_name, language, err))?;
        let enumerator = category
          .EnumTokens(None, None)
          .map_err(|err| SpeechError::into_speak_failed(synthesizer_name, language, err))?;
        let mut count: u32 = 0;
        enumerator
          .GetCount(&mut count)
          .map_err(|err| SpeechError::into_speak_failed(synthesizer_name, language, err))?;
        let mut tokens = Vec::with_capacity(count as _);
        let mut tokens_fetched: u32 = 0;
        enumerator
          .Next(count, tokens.as_mut_ptr(), Some(&mut tokens_fetched))
          .map_err(|err| SpeechError::into_speak_failed(synthesizer_name, language, err))?;
        tokens.set_len(tokens_fetched as _);
        tokens
          .into_iter()
          .flatten()
          .find(|token| {
            let closure = || {
              let attributes = token.OpenKey(w!("Attributes"))?;
              let lcid = attributes.GetStringValue(w!("Language"));
              let languages = match lcid {
                Ok(lcids) => lcids
                  .to_string()?
                  .split(';')
                  .flat_map(|lcid| {
                    let lcid = u32::from_str_radix(lcid, 16)?;
                    let mut name_vector = vec![0; LOCALE_NAME_MAX_LENGTH as _];
                    let length = LCIDToLocaleName(lcid, Some(&mut name_vector), 0);
                    name_vector.set_len((length - 1) as _);
                    Ok::<String, anyhow::Error>(String::from_utf16(&name_vector)?.to_lowercase())
                  })
                  .collect::<Vec<String>>(),
                _ => vec![],
              };
              Ok::<bool, anyhow::Error>(languages.iter().any(|name| name == language))
            };
            closure().unwrap_or(false)
          })
          .ok_or(SpeechError::into_language_not_found(language))?
      }
      (Some(voice), _) => {
        let token: ISpObjectToken = CoCreateInstance(&SpObjectToken, None, CLSCTX_ALL)
          .map_err(|err| SpeechError::into_speak_failed(synthesizer_name, voice, err))?;
        let mut voice_vector = voice.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
        token
          .SetId(
            SPCAT_VOICES,
            PWSTR::from_raw(voice_vector.as_mut_ptr()),
            false,
          )
          .map_err(|_| SpeechError::into_voice_not_found(voice))?;
        token
      }
    };
    synthesizer.SetVoice(&voice_token).map_err(|_| {
      SpeechError::into_voice_not_found(voice.unwrap_or(language.unwrap_or(default_voice)))
    })?;
    let rate = rate.unwrap_or(50) as i32;
    let rate = (rate / 5) - 10;
    synthesizer.SetRate(rate).map_err(|err| {
      SpeechError::into_speak_failed(
        synthesizer_name,
        voice.unwrap_or(language.unwrap_or(default_voice)),
        err,
      )
    })?;
    let volume = volume.unwrap_or(100) as u16;
    synthesizer.SetVolume(volume).map_err(|err| {
      SpeechError::into_speak_failed(
        synthesizer_name,
        voice.unwrap_or(language.unwrap_or(default_voice)),
        err,
      )
    })?;
    let pitch = pitch.unwrap_or(50) as i8;
    let pitch = (pitch / 5) - 10;
    let pitch = pitch.to_string();
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
      .create_element("pitch")
      .with_attribute(("absmiddle", pitch.as_str()))
      .write_text_content(BytesText::new(text))
      .map_err(SpeechError::into_unknown)?;
    let xml_vector = writer.into_inner().into_inner();
    String::from_utf8(xml_vector).map_err(SpeechError::into_unknown)
  }
}
pub struct Sapi {
  default_voice: String,
  stream_synthesizer: ISpVoice,
  playback_synthesizer: ISpVoice,
}
impl SpeechSynthesizer for Sapi {
  fn new() -> std::result::Result<Self, SpeechError> {
    unsafe {
      let stream_synthesizer: ISpVoice =
        CoCreateInstance(&SpVoice, None, CLSCTX_ALL).map_err(SpeechError::into_unknown)?;
      let playback_synthesizer: ISpVoice =
        CoCreateInstance(&SpVoice, None, CLSCTX_ALL).map_err(SpeechError::into_unknown)?;
      let default_voice = playback_synthesizer
        .GetVoice()
        .map_err(SpeechError::into_unknown)?
        .GetId()
        .map_err(SpeechError::into_unknown)?
        .to_string()
        .map_err(SpeechError::into_unknown)?;
      Ok(Sapi {
        default_voice,
        stream_synthesizer,
        playback_synthesizer,
      })
    }
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData {
      name: "SAPI 5".to_owned(),
      supports_to_audio_data: true,
      supports_to_audio_output: true,
      supports_speech_parameters: true,
    }
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, SpeechError> {
    unsafe {
      let category: ISpObjectTokenCategory =
        CoCreateInstance(&SpObjectTokenCategory, None, CLSCTX_ALL)
          .map_err(SpeechError::into_unknown)?;
      category
        .SetId(SPCAT_VOICES, false)
        .map_err(SpeechError::into_unknown)?;
      let enumerator = category
        .EnumTokens(None, None)
        .map_err(SpeechError::into_unknown)?;
      let mut count: u32 = 0;
      enumerator
        .GetCount(&mut count)
        .map_err(SpeechError::into_unknown)?;
      let mut tokens = Vec::with_capacity(count as _);
      let mut tokens_fetched: u32 = 0;
      enumerator
        .Next(count, tokens.as_mut_ptr(), Some(&mut tokens_fetched))
        .map_err(SpeechError::into_unknown)?;
      tokens.set_len(tokens_fetched as _);
      let voices = tokens
        .into_iter()
        .flatten()
        .flat_map(|token| {
          let name = token.GetId()?.to_string()?;
          let attributes = token.OpenKey(w!("Attributes"))?;
          let display_name = attributes.GetStringValue(w!("Name"));
          let lcid = attributes.GetStringValue(w!("Language"));
          let display_name = match display_name {
            Ok(display_name) => display_name.to_string()?,
            _ => "Unknown".to_owned(),
          };
          let mut seen = HashSet::new();
          let languages = match lcid {
            Ok(lcids) => lcids
              .to_string()?
              .split(';')
              .flat_map(|lcid| {
                let lcid = u32::from_str_radix(lcid, 16)?;
                let mut name_vector = vec![0; LOCALE_NAME_MAX_LENGTH as _];
                let length = LCIDToLocaleName(lcid, Some(&mut name_vector), 0);
                name_vector.set_len((length - 1) as _);
                Ok::<String, anyhow::Error>(String::from_utf16(&name_vector)?.to_lowercase())
              })
              .filter(|language| seen.insert(language.clone()))
              .collect::<Vec<String>>(),
            _ => vec![],
          };
          Ok::<Voice, anyhow::Error>(Voice {
            synthesizer: self.data(),
            display_name,
            name,
            languages,
            priority: 2,
          })
        })
        .collect::<Vec<Voice>>();
      Ok(voices)
    }
  }
  fn as_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    Some(self)
  }
  fn as_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    Some(self)
  }
}
impl SpeechSynthesizerToAudioData for Sapi {
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> std::result::Result<SpeechResult, SpeechError> {
    unsafe {
      let audio_stream = SHCreateMemStream(None).ok_or(SpeechError::into_unknown(anyhow!(
        "Failed to create memory stream",
      )))?;
      let formatted_stream: ISpStream =
        CoCreateInstance(&SpStream, None, CLSCTX_ALL).map_err(SpeechError::into_unknown)?;
      let format_guid = GUID::from_u128(0xc31adbae_527f_4ff5_a230_f62bb61ff70c);
      let format = WAVEFORMATEX {
        wFormatTag: WAVE_FORMAT_PCM as _,
        nChannels: 1,
        nSamplesPerSec: 44100,
        nAvgBytesPerSec: 88200,
        nBlockAlign: 2,
        wBitsPerSample: 16,
        cbSize: 0,
      };
      formatted_stream
        .SetBaseStream(&audio_stream, &format_guid, &format)
        .map_err(SpeechError::into_unknown)?;
      self
        .stream_synthesizer
        .SetOutput(&formatted_stream, false)
        .map_err(|err| {
          SpeechError::into_speak_failed(
            &self.data().name,
            voice.unwrap_or(language.unwrap_or(&self.default_voice)),
            err,
          )
        })?;
      let xml_string = set_parameters(
        &self.stream_synthesizer,
        &self.data().name,
        &self.default_voice,
        voice,
        language,
        rate,
        volume,
        pitch,
        text,
      )?;
      let mut xml = xml_string
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<u16>>();
      let flags = SPF_IS_XML.0 | SPF_PARSE_SAPI.0;
      self
        .stream_synthesizer
        .Speak(PWSTR::from_raw(xml.as_mut_ptr()), flags as u32, None)
        .map_err(|err| {
          SpeechError::into_speak_failed(
            &self.data().name,
            voice.unwrap_or(language.unwrap_or(&self.default_voice)),
            err,
          )
        })?;
      let mut pcm: Vec<u8> = Vec::new();
      let mut buffer: Vec<u8> = Vec::with_capacity(65536);
      let mut bytes_read: u32 = 0;
      formatted_stream
        .Seek(0, STREAM_SEEK_SET, None)
        .map_err(SpeechError::into_unknown)?;
      loop {
        let result = formatted_stream.Read(
          buffer.as_mut_ptr() as *mut c_void,
          65536,
          Some(&mut bytes_read),
        );
        if bytes_read == 0 {
          break;
        }
        buffer.set_len(bytes_read as _);
        pcm.append(&mut buffer);
        match result.ok() {
          Ok(()) => {}
          Err(_) => break,
        };
        buffer.clear();
      }
      Ok(SpeechResult {
        pcm,
        sample_format: SampleFormat::S16,
        sample_rate: 44100,
      })
    }
  }
}
impl SpeechSynthesizerToAudioOutput for Sapi {
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
    unsafe {
      let xml_string = set_parameters(
        &self.playback_synthesizer,
        &self.data().name,
        &self.default_voice,
        voice,
        language,
        rate,
        volume,
        pitch,
        text,
      )?;
      let mut xml = xml_string
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<u16>>();
      let flags = match interrupt {
        true => SPF_PURGEBEFORESPEAK.0 | SPF_ASYNC.0 | SPF_IS_XML.0 | SPF_PARSE_SAPI.0,
        false => SPF_ASYNC.0 | SPF_IS_XML.0 | SPF_PARSE_SAPI.0,
      };
      self
        .playback_synthesizer
        .Speak(PWSTR::from_raw(xml.as_mut_ptr()), flags as u32, None)
        .map_err(|err| {
          SpeechError::into_speak_failed(
            &self.data().name,
            voice.unwrap_or(language.unwrap_or(&self.default_voice)),
            err,
          )
        })?;
      Ok(())
    }
  }
  fn stop_speech(&self) -> std::result::Result<(), SpeechError> {
    unsafe {
      self
        .playback_synthesizer
        .Speak(None, SPF_PURGEBEFORESPEAK.0 as u32, None)
        .map_err(|err| SpeechError::into_stop_speech_failed(&self.data().name, err))?;
      Ok(())
    }
  }
}
