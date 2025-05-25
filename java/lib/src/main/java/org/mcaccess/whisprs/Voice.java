package org.mcaccess.whisprs;

public class Voice {
  public final SpeechSynthesizerMetadata synthesizer;
  public final String displayName;
  public final String name;
  public final String[] languages;
  public final byte priority;
  public Voice(SpeechSynthesizerMetadata synthesizer, String displayName, String name, String[] languages, byte priority) {
    this.synthesizer = synthesizer;
    this.displayName = displayName;
    this.name = name;
    this.languages = languages;
    this.priority = priority;
  }
}
