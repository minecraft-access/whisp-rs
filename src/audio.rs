#[derive(Clone, Debug)]
#[repr(u8)]
pub enum SampleFormat {
  S16 = 0,
  F32 = 1,
}
#[derive(Debug)]
pub struct SpeechResult {
  pub pcm: Vec<u8>,
  pub sample_format: SampleFormat,
  pub sample_rate: u32,
}
