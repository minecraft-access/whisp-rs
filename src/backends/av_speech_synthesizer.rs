use crate::audio::{SampleFormat, SpeechResult};
use crate::backends::{
  Backend, BrailleBackend, SpeechSynthesizerToAudioData, SpeechSynthesizerToAudioOutput,
};
use crate::error::OutputError;
use crate::metadata::Voice;
use anyhow::anyhow;
use block2::RcBlock;
use objc2::rc::Retained;
use objc2_avf_audio::{
  AVAudioBuffer, AVAudioCommonFormat, AVAudioPCMBuffer, AVSpeechBoundary, AVSpeechSynthesisVoice,
  AVSpeechSynthesisVoiceQuality, AVSpeechSynthesizer, AVSpeechUtterance,
  AVSpeechUtteranceMaximumSpeechRate, AVSpeechUtteranceMinimumSpeechRate,
};
use objc2_foundation::NSString;
use std::ptr::NonNull;
use std::sync::{mpsc, Arc, Mutex, OnceLock, RwLock};
fn set_parameters(
  voice: Option<&str>,
  language: Option<&str>,
  rate: Option<u8>,
  volume: Option<u8>,
  pitch: Option<u8>,
  text: &str,
) -> Result<Retained<AVSpeechUtterance>, OutputError> {
  unsafe {
    let text = NSString::from_str(text);
    let utterance = AVSpeechUtterance::speechUtteranceWithString(&text);
    match (voice, language) {
      (None, None) => {}
      (Some(voice_name), _) => {
        let voice = NSString::from_str(voice_name);
        let voice = AVSpeechSynthesisVoice::voiceWithIdentifier(&voice)
          .ok_or(OutputError::into_voice_not_found(voice_name))?;
        utterance.setVoice(Some(&voice));
      }
      (_, Some(language)) => {
        let voice = AVSpeechSynthesisVoice::speechVoices()
          .into_iter()
          .find(|voice| voice.language().to_string().to_lowercase() == language)
          .ok_or(OutputError::into_language_not_found(language))?;
        utterance.setVoice(Some(&voice));
      }
    };
    let minimum_rate: f32 = AVSpeechUtteranceMinimumSpeechRate;
    let maximum_rate: f32 = AVSpeechUtteranceMaximumSpeechRate;
    let rate = f32::from(rate.unwrap_or(50));
    let rate = (rate / 100.0) * (maximum_rate - minimum_rate) + minimum_rate;
    utterance.setRate(rate);
    let volume = f32::from(volume.unwrap_or(100));
    let volume = volume / 100.0;
    utterance.setVolume(volume);
    let pitch = f32::from(pitch.unwrap_or(50));
    let pitch = pitch / 100.0;
    let pitch = if pitch < 0.5 {
      pitch * 2.0 * 0.75 + 0.25
    } else {
      pitch * 2.0
    };
    utterance.setPitchMultiplier(pitch);
    Ok(utterance)
  }
}
pub struct AvSpeechSynthesizer {
  synthesizer: Mutex<Retained<AVSpeechSynthesizer>>,
}
impl Backend for AvSpeechSynthesizer {
  fn new() -> Result<Self, OutputError> {
    let result = AvSpeechSynthesizer {
      synthesizer: unsafe { Mutex::new(AVSpeechSynthesizer::new()) },
    };
    Ok(result)
  }
  fn name(&self) -> String {
    "AVSpeechSynthesizer".to_owned()
  }
  fn list_voices(&self) -> Result<Vec<Voice>, OutputError> {
    unsafe {
      let voices = AVSpeechSynthesisVoice::speechVoices();
      let voices = voices
        .into_iter()
        .map(|voice| {
          let languages = vec![voice.language().to_string().to_lowercase()];
          let name = voice.identifier().to_string();
          let display_name = voice.name().to_string();
          let quality = voice.quality();
          let display_name = match quality {
            AVSpeechSynthesisVoiceQuality::Enhanced => display_name + " (Enhanced)",
            AVSpeechSynthesisVoiceQuality::Premium => display_name + " (Premium)",
            _ => display_name,
          };
          let priority: u8 = match quality {
            AVSpeechSynthesisVoiceQuality::Premium => 1,
            AVSpeechSynthesisVoiceQuality::Enhanced => 2,
            _ => 3,
          };
          Voice {
            synthesizer: self.speech_metadata().unwrap(),
            display_name,
            name,
            languages,
            priority,
          }
        })
        .collect::<Vec<Voice>>();
      Ok(voices)
    }
  }
  fn as_speech_synthesizer_to_audio_data(&self) -> Option<&dyn SpeechSynthesizerToAudioData> {
    Some(self)
  }
  fn as_speech_synthesizer_to_audio_output(&self) -> Option<&dyn SpeechSynthesizerToAudioOutput> {
    Some(self)
  }
  fn as_braille_backend(&self) -> Option<&dyn BrailleBackend> {
    None
  }
}
impl SpeechSynthesizerToAudioData for AvSpeechSynthesizer {
  fn supports_speech_parameters(&self) -> bool {
    true
  }
  #[allow(clippy::cast_possible_truncation)]
  #[allow(clippy::cast_sign_loss)]
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
  ) -> Result<SpeechResult, OutputError> {
    unsafe {
      let utterance = set_parameters(voice, language, rate, volume, pitch, text)?;
      let pcm: Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(Vec::new()));
      let pcm2 = pcm.clone();
      let sample_format: Arc<OnceLock<SampleFormat>> = Arc::new(OnceLock::new());
      let sample_format2 = sample_format.clone();
      let sample_rate: Arc<OnceLock<u32>> = Arc::new(OnceLock::new());
      let sample_rate2 = sample_rate.clone();
      let (done_tx, done_rx) = mpsc::channel::<Result<(), OutputError>>();
      let callback = RcBlock::new(move |buffer: NonNull<AVAudioBuffer>| {
        let closure =
          || {
            let buffer = buffer.as_ref().downcast_ref::<AVAudioPCMBuffer>().ok_or(
              OutputError::into_unknown(anyhow!("AVSpeechSynthesizer did not return a PCM buffer")),
            )?;
            let format = buffer.format();
            let sample_format = match format.commonFormat() {
              AVAudioCommonFormat::PCMFormatFloat32 => SampleFormat::F32,
              AVAudioCommonFormat::PCMFormatInt16 => SampleFormat::S16,
              _ => Err(OutputError::into_unknown(anyhow!(
                "Invalid audio format from AVSpeechSynthesizer"
              )))?,
            };
            let frame_length = buffer.frameLength();
            if frame_length > 0 {
              let sample_size = match sample_format {
                SampleFormat::F32 => 4,
                SampleFormat::S16 => 2,
              };
              let mut data = match sample_format {
                SampleFormat::F32 => (*buffer.floatChannelData()).as_ptr() as *const u8,
                SampleFormat::S16 => (*buffer.int16ChannelData()).as_ptr() as *const u8,
              };
              let stride = buffer.stride() * sample_size;
              let mut pcm2 = pcm2
                .write()
                .map_err(|_| OutputError::into_unknown(anyhow!("Failed to write PCM vector")))?;
              for _ in 0..frame_length - 1 {
                let mut sample = std::slice::from_raw_parts(data, sample_size).to_vec();
                pcm2.append(&mut sample);
                data = data.add(stride);
              }
            } else {
              sample_format2
                .set(sample_format)
                .map_err(|_| OutputError::into_unknown(anyhow!("Failed to set sample format")))?;
              sample_rate2
                .set(format.sampleRate() as u32)
                .map_err(|_| OutputError::into_unknown(anyhow!("Failed to set sample rate")))?;
            };
            Ok(())
          };
        done_tx.send(closure()).unwrap();
      });
      self
        .synthesizer
        .lock()
        .map_err(|_| {
          OutputError::into_unknown(anyhow!("Failed to lock AVSpeechSynthesizer instance"))
        })?
        .writeUtterance_toBufferCallback(&utterance, RcBlock::as_ptr(&callback));
      done_rx.recv().map_err(OutputError::into_unknown)??;
      let pcm = pcm
        .read()
        .map_err(|_| OutputError::into_unknown(anyhow!("Failed to read PCM vector")))?
        .clone();
      let sample_format = sample_format
        .get()
        .ok_or(OutputError::into_unknown(anyhow!(
          "Sample format not set".to_owned()
        )))?
        .to_owned();
      let sample_rate = sample_rate
        .get()
        .ok_or(OutputError::into_unknown(anyhow!(
          "Sample rate not set".to_owned()
        )))?
        .to_owned();
      Ok(SpeechResult {
        pcm,
        sample_format,
        sample_rate,
      })
    }
  }
}
impl SpeechSynthesizerToAudioOutput for AvSpeechSynthesizer {
  fn supports_speech_parameters(&self) -> bool {
    true
  }
  fn speak(
    &self,
    voice: Option<&str>,
    language: Option<&str>,
    rate: Option<u8>,
    volume: Option<u8>,
    pitch: Option<u8>,
    text: &str,
    interrupt: bool,
  ) -> Result<(), OutputError> {
    unsafe {
      let utterance = set_parameters(voice, language, rate, volume, pitch, text)?;
      if interrupt {
        self
          .synthesizer
          .lock()
          .map_err(|_| {
            OutputError::into_unknown(anyhow!("Failed to lock AVSpeechSynthesizer instance"))
          })?
          .stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
      };
      self
        .synthesizer
        .lock()
        .map_err(|_| {
          OutputError::into_unknown(anyhow!("Failed to lock AVSpeechSynthesizer instance"))
        })?
        .speakUtterance(&utterance);
      Ok(())
    }
  }
  fn stop_speech(&self) -> Result<(), OutputError> {
    unsafe {
      self
        .synthesizer
        .lock()
        .map_err(|_| {
          OutputError::into_unknown(anyhow!("Failed to lock AVSpeechSynthesizer instance"))
        })?
        .stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
      Ok(())
    }
  }
}
