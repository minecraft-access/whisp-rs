package org.mcaccess.whisprs.metadata;

import org.jetbrains.annotations.NotNull;

public class SpeechSynthesizerMetadata {
  public final @NotNull String name;
  public final boolean supportsSpeakingToAudioData;
  public final boolean supportsSpeechParameters;
  public SpeechSynthesizerMetadata(@NotNull String name, boolean supportsSpeakingToAudioData, boolean supportsSpeechParameters) {
    this.name = name;
    this.supportsSpeakingToAudioData = supportsSpeakingToAudioData;
    this.supportsSpeechParameters = supportsSpeechParameters;
  }
}
