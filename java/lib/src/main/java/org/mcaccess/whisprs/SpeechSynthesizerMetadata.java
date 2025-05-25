package org.mcaccess.whisprs;

public class SpeechSynthesizerMetadata {
  public final String name;
  public final boolean supportsSpeakingToAudioData;
  public final boolean supportsSpeechParameters;
  public SpeechSynthesizerMetadata(String name, boolean supportsSpeakingToAudioData, boolean supportsSpeechParameters) {
    this.name = name;
    this.supportsSpeakingToAudioData = supportsSpeakingToAudioData;
    this.supportsSpeechParameters = supportsSpeechParameters;
  }
}
