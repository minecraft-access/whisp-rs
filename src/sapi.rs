use std::collections::HashSet;
use std::ffi::c_void;
use std::io::Cursor;
use quick_xml::events::BytesText;
use quick_xml::writer::Writer;
use windows::core::*;
use windows::Win32::Globalization::LCIDToLocaleName;
use windows::Win32::Media::Audio::*;
use windows::Win32::Media::Speech::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::SystemServices::LOCALE_NAME_MAX_LENGTH;
use windows::Win32::UI::Shell::SHCreateMemStream;
use crate::speech_synthesizer::*;
pub struct Sapi {
  stream_synthesizer: ISpVoice,
  playback_synthesizer: ISpVoice
}
impl SpeechSynthesizer for Sapi {
  fn new() -> std::result::Result<Self, SpeechError> {
    unsafe {
      let stream_synthesizer: ISpVoice = CoCreateInstance(&SpVoice, None, CLSCTX_ALL).unwrap();
      let playback_synthesizer: ISpVoice = CoCreateInstance(&SpVoice, None, CLSCTX_ALL).unwrap();
      Ok(Sapi { stream_synthesizer, playback_synthesizer })
    }
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData { name: "SAPI 5".to_owned(), supports_to_audio_data: true, supports_to_audio_output: true, supports_speech_parameters: true }
  }
  fn list_voices(&self) -> std::result::Result<Vec<Voice>, SpeechError> {
    unsafe {
      let category: ISpObjectTokenCategory = CoCreateInstance(&SpObjectTokenCategory, None, CLSCTX_ALL).unwrap();
      category.SetId(SPCAT_VOICES, false).unwrap();
      let enumerator = category.EnumTokens(None, None).unwrap();
      let mut count: u32 = 0;
      enumerator.GetCount(&mut count).unwrap();
      let mut tokens = Vec::with_capacity(count.try_into().unwrap());
      let mut tokens_fetched: u32 = 0;
      enumerator.Next(count, tokens.as_mut_ptr(), Some(&mut tokens_fetched)).unwrap();
      tokens.set_len(tokens_fetched.try_into().unwrap());
      let voices = tokens
        .into_iter()
        .filter_map(std::convert::identity)
        .map(|token| {
          let name = token.GetId().unwrap().to_string().unwrap();
          let attributes = token.OpenKey(w!("Attributes")).unwrap();
          let display_name = attributes.GetStringValue(w!("Name"));
          let lcid = attributes.GetStringValue(w!("Language"));
          let display_name = match display_name {
            Ok(display_name) => display_name.to_string().unwrap(),
            _ => "Unknown".to_owned()
          };
          let mut seen = HashSet::new();
          let languages = match lcid {
            Ok(lcids) => lcids
              .to_string()
              .unwrap()
              .split(';')
              .map(|lcid| {
                let lcid = u32::from_str_radix(lcid, 16).unwrap();
                let mut name_vector = Vec::with_capacity(LOCALE_NAME_MAX_LENGTH.try_into().unwrap());
                name_vector.set_len(LOCALE_NAME_MAX_LENGTH.try_into().unwrap());
                let length = LCIDToLocaleName(lcid, Some(&mut name_vector), 0);
                name_vector.set_len((length-1).try_into().unwrap());
                String::from_utf16(&name_vector).unwrap().to_lowercase()
              })
              .filter(|language| seen.insert(language.clone()))
              .collect::<Vec<String>>(),
            _ => vec!()
          };
          Voice { synthesizer: self.data(), display_name, name, languages, priority: 2 }
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
  fn speak(&self, voice: &str, _language: &str, rate: Option<u8>, volume: Option<u8>, pitch: Option<u8>, text: &str) -> std::result::Result<SpeechResult, SpeechError> {
    unsafe {
      let audio_stream = SHCreateMemStream(None).unwrap();
      let formatted_stream: ISpStream = CoCreateInstance(&SpStream, None, CLSCTX_ALL).unwrap();
      let format_guid = GUID::from_u128(0xc31adbae_527f_4ff5_a230_f62bb61ff70c);
      let format = WAVEFORMATEX { wFormatTag: WAVE_FORMAT_PCM as _, nChannels: 1, nSamplesPerSec: 44100, nAvgBytesPerSec: 88200, nBlockAlign: 2, wBitsPerSample: 16, cbSize: 0 };
      formatted_stream.SetBaseStream(&audio_stream, &format_guid, &format).unwrap();
      self.stream_synthesizer.SetOutput(&formatted_stream, false).unwrap();
      let voice_token: ISpObjectToken = CoCreateInstance(&SpObjectToken, None, CLSCTX_ALL).unwrap();
      let mut voice = voice.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
      voice_token.SetId(SPCAT_VOICES, PWSTR::from_raw(voice.as_mut_ptr()), false)?;
      self.stream_synthesizer.SetVoice(&voice_token)?;
      let rate = rate.unwrap_or(50) as i32;
      let rate = (rate/5)-10;
      self.stream_synthesizer.SetRate(rate)?;
      let volume = volume.unwrap_or(100) as u16;
      self.stream_synthesizer.SetVolume(volume)?;
      let pitch = pitch.unwrap_or(50) as i8;
      let pitch = (pitch/5)-10;
      let pitch = pitch.to_string();
      let mut writer = Writer::new(Cursor::new(Vec::new()));
      writer.create_element("pitch")
        .with_attribute(("absmiddle", pitch.as_str()))
        .write_text_content(BytesText::new(text))
        .unwrap();
      let xml_vector = writer.into_inner().into_inner();
      let xml_string = String::from_utf8(xml_vector).unwrap();
      let mut xml = xml_string.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
      let flags = SPF_IS_XML.0 | SPF_PARSE_SAPI.0;
      self.stream_synthesizer.Speak(PWSTR::from_raw(xml.as_mut_ptr()), flags as u32, None)?;
      let mut pcm: Vec<u8> = Vec::new();
      let mut buffer: Vec<u8> = Vec::with_capacity(65536);
      let mut bytes_read: u32 = 0;
      formatted_stream.Seek(0, STREAM_SEEK_SET, None).unwrap();
      loop {
        let result = formatted_stream.Read(buffer.as_mut_ptr() as *mut c_void, 65536, Some(&mut bytes_read));
        if bytes_read==0 {
          break
        }
        buffer.set_len(bytes_read.try_into().unwrap());
        pcm.append(&mut buffer);
        match result.ok() {
          Ok(()) => {},
          Err(_) => break
        };
        buffer.clear();
      }
      Ok(SpeechResult { pcm, sample_format: SampleFormat::S16, sample_rate: 44100 })
    }
  }
}
impl SpeechSynthesizerToAudioOutput for Sapi {
  fn speak(&self, voice: &str, _language: &str, rate: Option<u8>, volume: Option<u8>, pitch: Option<u8>, text: &str, interrupt: bool) -> std::result::Result<(), SpeechError> {
    unsafe {
      let voice_token: ISpObjectToken = CoCreateInstance(&SpObjectToken, None, CLSCTX_ALL).unwrap();
      let mut voice = voice.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
      voice_token.SetId(SPCAT_VOICES, PWSTR::from_raw(voice.as_mut_ptr()), false)?;
      self.playback_synthesizer.SetVoice(&voice_token)?;
      let rate = rate.unwrap_or(50) as i32;
      let rate = (rate/5)-10;
      self.playback_synthesizer.SetRate(rate)?;
      let volume = volume.unwrap_or(100) as u16;
      self.playback_synthesizer.SetVolume(volume)?;
      let pitch = pitch.unwrap_or(50) as i8;
      let pitch = (pitch/5)-10;
      let pitch = pitch.to_string();
      let mut writer = Writer::new(Cursor::new(Vec::new()));
      writer.create_element("pitch")
        .with_attribute(("absmiddle", pitch.as_str()))
        .write_text_content(BytesText::new(text))
        .unwrap();
      let xml_vector = writer.into_inner().into_inner();
      let xml_string = String::from_utf8(xml_vector).unwrap();
      let mut xml = xml_string.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
      let flags = match interrupt {
        true => SPF_PURGEBEFORESPEAK.0 | SPF_ASYNC.0 | SPF_IS_XML.0 | SPF_PARSE_SAPI.0,
        false => SPF_ASYNC.0 | SPF_IS_XML.0 | SPF_PARSE_SAPI.0
      };
      self.playback_synthesizer.Speak(PWSTR::from_raw(xml.as_mut_ptr()), flags as u32, None)?;
      Ok(())
    }
  }
  fn stop_speech(&self) -> std::result::Result<(), SpeechError> {
    unsafe {
      self.playback_synthesizer.Speak(None, SPF_PURGEBEFORESPEAK.0 as u32, None)?;
      Ok(())
    }
  }
}
