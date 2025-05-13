use std::ptr::NonNull;
use std::sync::{Arc,Mutex,mpsc,OnceLock,RwLock};
use objc2::rc::Retained;
use objc2_foundation::{NSDate,NSRunLoop,NSString};
use block2::RcBlock;
use objc2_avf_audio::{AVAudioBuffer,AVAudioPCMBuffer,AVAudioCommonFormat,AVSpeechBoundary,AVSpeechSynthesisVoice,AVSpeechSynthesisVoiceQuality,AVSpeechSynthesizer,AVSpeechUtterance,AVSpeechUtteranceMaximumSpeechRate,AVSpeechUtteranceMinimumSpeechRate};
use crate::speech_synthesizer::*;
fn run_run_loop(duration: f64) {
  unsafe {
    let run_loop = NSRunLoop::currentRunLoop();
    let date = NSDate::now().dateByAddingTimeInterval(duration);
    run_loop.runUntilDate(&date);
  }
}
#[derive(Debug)] pub struct AvSpeechSynthesizer {
  synthesizer: Mutex<Retained<AVSpeechSynthesizer>>
}
impl SpeechSynthesizer for AvSpeechSynthesizer {
  fn new() -> Result<Self, SpeechError> {
    let result = AvSpeechSynthesizer { synthesizer: unsafe { Mutex::new(AVSpeechSynthesizer::new()) }};
    run_run_loop(0.1);
    Ok(result)
  }
  fn data(&self) -> SpeechSynthesizerData {
    SpeechSynthesizerData { name: "AVSpeechSynthesizer".to_owned(), supports_to_audio_data: true, supports_to_audio_output: true, supports_speech_parameters: true }
  }
  fn list_voices(&self) -> Result<Vec<Voice>, SpeechError> {
    unsafe {
      run_run_loop(0.1);
      let voices = AVSpeechSynthesisVoice::speechVoices();
      let voices = voices.iter()
        .map(|voice| {
          let languages = vec!(voice.language().to_string().to_lowercase());
          let name = voice.identifier().to_string();
          let display_name = voice.name().to_string();
          let quality = voice.quality();
          let display_name = match quality {
            AVSpeechSynthesisVoiceQuality::Enhanced => display_name+" (Enhanced)",
            AVSpeechSynthesisVoiceQuality::Premium => display_name+" (Premium)",
            _ => display_name,
          };
          let priority: u8 = match quality {
            AVSpeechSynthesisVoiceQuality::Premium => 1,
            AVSpeechSynthesisVoiceQuality::Enhanced => 2,
            _ => 3,
          };
          Voice { synthesizer: self.data(), display_name, name, languages, priority }
        })
        .collect::<Vec<Voice>>();
      run_run_loop(0.1);
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
impl SpeechSynthesizerToAudioData for AvSpeechSynthesizer {
  fn speak(&self, voice: &str, _language: &str, rate: Option<u8>, volume: Option<u8>, pitch: Option<u8>, text: &str) -> Result<SpeechResult, SpeechError> {
    unsafe {
      run_run_loop(0.1);
      let text = NSString::from_str(text);
      let utterance = AVSpeechUtterance::speechUtteranceWithString(&text);
      let voice = NSString::from_str(voice);
      let voice = AVSpeechSynthesisVoice::voiceWithIdentifier(&voice).ok_or(SpeechError { message: "No AVSpeechSynthesizer voices found with this name".to_owned() })?;
      utterance.setVoice(Some(&voice));
      let minimum_rate: f32 = AVSpeechUtteranceMinimumSpeechRate;
      let maximum_rate: f32 = AVSpeechUtteranceMaximumSpeechRate;
      let rate = rate.unwrap_or(50) as f32;
      let rate = (rate/100.0)*(maximum_rate-minimum_rate)+minimum_rate;
      utterance.setRate(rate);
      let volume = volume.unwrap_or(100) as f32;
      let volume = volume/100.0;
      utterance.setVolume(volume);
      let pitch = pitch.unwrap_or(50) as f32;
      let pitch = pitch/100.0;
      let pitch = if pitch<0.5 { pitch*2.0*0.75+0.25 } else { pitch*2.0 };
      utterance.setPitchMultiplier(pitch);
      let pcm: Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(Vec::new()));
      let pcm2 = pcm.clone();
      let sample_format: Arc<OnceLock<SampleFormat>> = Arc::new(OnceLock::new());
      let sample_format2 = sample_format.clone();
      let sample_rate: Arc<OnceLock<u32>> = Arc::new(OnceLock::new());
      let sample_rate2 = sample_rate.clone();
      let (done_tx, done_rx) = mpsc::channel();
      let callback = RcBlock::new(move |buffer: NonNull<AVAudioBuffer>| {
        let buffer = buffer.as_ref().downcast_ref::<AVAudioPCMBuffer>().expect("AVSpeechSynthesizer did not return a PCM buffer");
        let format = buffer.format();
        let sample_format = match format.commonFormat() {
          AVAudioCommonFormat::PCMFormatFloat32 => SampleFormat::F32,
          AVAudioCommonFormat::PCMFormatInt16 => SampleFormat::S16,
          _ => panic!("Invalid audio format from AVSpeechSynthesizer")
        };
        let frame_length = buffer.frameLength();
        if frame_length>0 {
          let sample_size = match sample_format {
            SampleFormat::F32 => 4,
            SampleFormat::S16 => 2
          };
          let mut data = match sample_format {
            SampleFormat::F32 => (*buffer.floatChannelData()).as_ptr() as *const u8,
            SampleFormat::S16 => (*buffer.int16ChannelData()).as_ptr() as *const u8
          };
          let stride = buffer.stride()*sample_size;
          let mut pcm2 = pcm2.write().unwrap();
            for _ in 0..frame_length-1 {
            let mut sample = std::slice::from_raw_parts(data, sample_size)
              .into_iter()
              .map(|byte| byte.clone())
              .collect::<Vec<u8>>();
            pcm2.append(&mut sample);
            data = data.add(stride);
          };
        }
        else {
          sample_format2.set(sample_format).unwrap();
          sample_rate2.set(format.sampleRate() as u32).unwrap();
          done_tx.send(()).unwrap();
        };
      });
      self.synthesizer.lock()?.writeUtterance_toBufferCallback(&utterance, RcBlock::as_ptr(&callback));
      loop {
        run_run_loop(0.1);
        match done_rx.try_recv() {
          Ok(()) => break,
          Err(mpsc::TryRecvError::Empty) => continue,
          Err(mpsc::TryRecvError::Disconnected) => return Err(SpeechError { message: "Channel disconnected".to_owned() }),
        };
      };
      let pcm = pcm.read()?.clone();
      let sample_format = sample_format.get().ok_or(SpeechError { message: "Sample format not set".to_owned() })?.to_owned();
      let sample_rate = sample_rate.get().ok_or(SpeechError { message: "Sample rate not set".to_owned() })?.to_owned();
      Ok(SpeechResult { pcm, sample_format, sample_rate })
    }
  }
}
impl SpeechSynthesizerToAudioOutput for AvSpeechSynthesizer {
  fn speak(&self, voice: &str, _language: &str, rate: Option<u8>, volume: Option<u8>, pitch: Option<u8>, text: &str, interrupt: bool) -> Result<(), SpeechError> {
    unsafe {
      run_run_loop(0.1);
      let text = NSString::from_str(text);
      let utterance = AVSpeechUtterance::speechUtteranceWithString(&text);
      let voice = NSString::from_str(voice);
      let voice = AVSpeechSynthesisVoice::voiceWithIdentifier(&voice).ok_or(SpeechError { message: "No AVSpeechSynthesizer voices found with this name".to_owned() })?;
      utterance.setVoice(Some(&voice));
      let minimum_rate: f32 = AVSpeechUtteranceMinimumSpeechRate;
      let maximum_rate: f32 = AVSpeechUtteranceMaximumSpeechRate;
      let rate = rate.unwrap_or(50) as f32;
      let rate = (rate/100.0)*(maximum_rate-minimum_rate)+minimum_rate;
      utterance.setRate(rate);
      let volume = volume.unwrap_or(100) as f32;
      let volume = volume/100.0;
      utterance.setVolume(volume);
      let pitch = pitch.unwrap_or(50) as f32;
      let pitch = pitch/100.0;
      let pitch = if pitch<0.5 { pitch*2.0*0.75+0.25 } else { pitch*2.0 };
      utterance.setPitchMultiplier(pitch);
      if interrupt {
        self.synthesizer.lock()?.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
      };
      self.synthesizer.lock()?.speakUtterance(&utterance);
      Ok(())
    }
  }
  fn stop_speech(&self) -> Result<(), SpeechError> {
    unsafe {
      self.synthesizer.lock()?.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
      Ok(())
    }
  }
}
