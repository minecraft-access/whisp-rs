package org.mcaccess.whisprs;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

public class Whisprs {
  public static native void initialize();
  public static native @NotNull Voice[] listVoices();
  public static native @NotNull BrailleBackendMetadata[] listBrailleBackends();
  public static native @NotNull SpeechResult speakToAudioData(@Nullable String synthesizer, @Nullable String voice, @Nullable String language, @Nullable Byte rate, @Nullable Byte volume, @Nullable Byte pitch, @NotNull String text);
  public static native void speakToAudioOutput(@Nullable String synthesizer, @Nullable String voice, @Nullable String language, @Nullable Byte rate, @Nullable Byte volume, @Nullable Byte pitch, @NotNull String text, boolean interrupt);
  public static native void stopSpeech(@Nullable String synthesizer);
  public static native void braille(@Nullable String synthesizer, @NotNull String text);
  public static native void output(@Nullable String synthesizer, @Nullable String voice, @Nullable String language, @Nullable Byte rate, @Nullable Byte volume, @Nullable Byte pitch, @Nullable String brailleBackend, @NotNull String text, boolean interrupt);
}
