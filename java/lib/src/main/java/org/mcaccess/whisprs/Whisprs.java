package org.mcaccess.whisprs;

public class Whisprs {
  public static native void initialize();
  public static native Voice[] listVoices();
  public static native BrailleBackendMetadata[] listBrailleBackends();
  public static native SpeechResult speakToAudioData(String synthesizer, String voice, String language, Byte rate, Byte volume, Byte pitch, String text);
  public static native void speakToAudioOutput(String synthesizer, String voice, String language, Byte rate, Byte volume, Byte pitch, String text, boolean interrupt);
  public static native void stopSpeech(String synthesizer);
  public static native void braille(String synthesizer, String text);
  public static native void output(String synthesizer, String voice, String language, Byte rate, Byte volume, Byte pitch, String brailleBackend, String text, boolean interrupt);
}
