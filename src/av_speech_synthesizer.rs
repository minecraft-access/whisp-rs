use std::ptr::NonNull;
use std::sync::{Arc,RwLock};
use objc2::rc::Retained;
use objc2_foundation::NSString;
use block2::RcBlock;
use objc2_avf_audio::{AVAudioBuffer,AVSpeechSynthesisVoice,AVSpeechSynthesizer,AVSpeechUtterance,AVSpeechUtteranceMaximumSpeechRate,AVSpeechUtteranceMinimumSpeechRate};
use crate::speech_synthesizer::{SpeechError,SpeechResult,SpeechSynthesizer,Voice};
#[derive(Debug)] pub struct AvSpeechSynthesizer {
  synthesizer: Retained<AVSpeechSynthesizer>
}
impl SpeechSynthesizer for AvSpeechSynthesizer {
  fn new() -> Result<Self, SpeechError> {
    Ok(AvSpeechSynthesizer { synthesizer: unsafe { AVSpeechSynthesizer::new() }})
  }
  fn name(&self) -> String {
    "AVSpeechSynthesizer".to_owned()
  }
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError> {
    unsafe {
      let voices = AVSpeechSynthesisVoice::speechVoices();
      let voices = voices.iter()
        .map(|voice| {
          let language = voice.language().to_string();
          let identifier = voice.identifier().to_string();
          let name = voice.name().to_string();
          Voice { synthesizer: self.name(), display_name: name, name: identifier, language }
        })
        .collect::<Vec<Voice>>();
      Ok(voices)
    }
  }
  fn speak(&self, voice: &str, _language: &str, rate: u8, volume: u8, pitch: u8, text: &str) -> Result<SpeechResult, SpeechError> {
    unsafe {
      let text = NSString::from_str(text);
      let utterance = AVSpeechUtterance::speechUtteranceWithString(&text);
      let voice = NSString::from_str(voice);
      let voice = AVSpeechSynthesisVoice::voiceWithIdentifier(&voice).ok_or(SpeechError { message: "No AVSpeechSynthesizer voices found with this name".to_owned() })?;
      utterance.setVoice(Some(&voice));
      let minimum_rate: f32 = AVSpeechUtteranceMinimumSpeechRate;
      let maximum_rate: f32 = AVSpeechUtteranceMaximumSpeechRate;
      let rate = rate as f32;
      let rate = (rate/100.0)*(maximum_rate-minimum_rate)+minimum_rate;
      utterance.setRate(rate);
      let pitch = pitch as f32;
      let pitch = pitch/100.0;
      let pitch = if pitch<0.5 { pitch*2.0*0.75+0.25 } else { pitch*2.0 };
      utterance.setPitchMultiplier(pitch);
      let volume = volume as f32;
      let volume = volume/100.0;
      utterance.setVolume(volume);
      let pcm: Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(Vec::new()));
      let pcm2 = pcm.clone();
      let sample_rate: Arc<RwLock<u32>> = Arc::new(RwLock::new(0));
      let sample_rate2 = sample_rate.clone();
      let callback = RcBlock::new(move |buffer: NonNull<AVAudioBuffer>| {
        let buffers = buffer.as_ref().audioBufferList().as_ref();
        let buffers = std::slice::from_raw_parts(buffers.mBuffers.as_ptr(), buffers.mNumberBuffers as usize);
        buffers.into_iter().for_each(|buffer| {
          let buffer = std::slice::from_raw_parts(buffer.mData as *const u8, buffer.mDataByteSize as usize);
          let mut buffer = buffer.into_iter().map(|byte| byte.clone()).collect::<Vec<u8>>();
          pcm2.write().unwrap().append(&mut buffer);
        });
      });
      self.synthesizer.writeUtterance_toBufferCallback(&utterance, RcBlock::into_raw(callback));
      let pcm = pcm.read()?.clone();
      Ok(SpeechResult { pcm, sample_rate: 0 })
    }
  }
}
