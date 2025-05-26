package org.mcaccess.whisprs;

import org.jetbrains.annotations.NotNull;

public class Voice {
  public final @NotNull SpeechSynthesizerMetadata synthesizer;
  public final @NotNull String displayName;
  public final @NotNull String name;
  public final @NotNull String[] languages;
  public final byte priority;
  public Voice(@NotNull SpeechSynthesizerMetadata synthesizer, @NotNull String displayName, @NotNull String name, @NotNull String[] languages, byte priority) {
    this.synthesizer = synthesizer;
    this.displayName = displayName;
    this.name = name;
    this.languages = languages;
    this.priority = priority;
  }
}
