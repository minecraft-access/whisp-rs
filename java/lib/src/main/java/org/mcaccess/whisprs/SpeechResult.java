package org.mcaccess.whisprs;

public class SpeechResult {
  enum SampleFormat {
    S16, F32
  }
  public final byte[] pcm;
  public final SampleFormat sampleFormat;
  public final int sampleRate;
  public SpeechResult(byte[] pcm, byte sampleFormat, int sampleRate) {
    this.pcm = pcm;
    this.sampleFormat = SampleFormat.values()[(int) sampleFormat];
    this.sampleRate = sampleRate;
  }
}
