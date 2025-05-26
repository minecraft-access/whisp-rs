package org.mcaccess.whisprs;

import org.jetbrains.annotations.NotNull;

public class SpeechResult {
  enum SampleFormat {
    S16, F32
  }
  public final @NotNull byte[] pcm;
  public final @NotNull SampleFormat sampleFormat;
  public final int sampleRate;
  public SpeechResult(@NotNull byte[] pcm, byte sampleFormat, int sampleRate) {
    this.pcm = pcm;
    this.sampleFormat = SampleFormat.values()[(int) sampleFormat];
    this.sampleRate = sampleRate;
  }
}
