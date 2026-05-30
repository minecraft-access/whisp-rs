package org.mcaccess.whisprs;

import java.lang.ref.Cleaner;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;
import org.mcaccess.whisprs.audio.SpeechResult;
import org.mcaccess.whisprs.metadata.BrailleBackendMetadata;
import org.mcaccess.whisprs.metadata.SpeechSynthesizerMetadata;
import org.mcaccess.whisprs.metadata.Voice;

public class Whisprs implements AutoCloseable {
  private static final Cleaner CLEANER = Cleaner.create();
  private long handle;
  private final Cleaner.Cleanable cleanable;
  public Whisprs() {
    long handle = create();
    this.handle = handle;
    this.cleanable = CLEANER.register(this, new DestroyAction(handle));
  }
  @Override
  public synchronized void close() {
    handle = 0;
    cleanable.clean();
  }
  private static native long create();
  private static native void destroy(long handle);
  private static final class DestroyAction implements Runnable {
    private final long handle;
    DestroyAction(long handle) {
      this.handle = handle;
    }
    @Override
    public void run() {
      if (handle != 0) {
        destroy(handle);
      }
    }
  }
  public native @NotNull Voice[] listVoices(@Nullable String synthesizer, @Nullable String voice, @Nullable String language, boolean needsAudioData);
  public native @NotNull SpeechSynthesizerMetadata[] listSpeechSynthesizers();
  public native @NotNull SpeechSynthesizerMetadata[] listSpeechSynthesizersSupportingAudioData();
  public native @NotNull BrailleBackendMetadata[] listBrailleBackends();
  public native @NotNull SpeechResult speakToAudioData(@Nullable String synthesizer, @Nullable String voice, @Nullable String language, @Nullable Byte rate, @Nullable Byte volume, @Nullable Byte pitch, @NotNull String text);
  public native void speakToAudioOutput(@Nullable String synthesizer, @Nullable String voice, @Nullable String language, @Nullable Byte rate, @Nullable Byte volume, @Nullable Byte pitch, @NotNull String text, boolean interrupt);
  public native void stopSpeech(@Nullable String synthesizer);
  public native void braille(@Nullable String synthesizer, @NotNull String text);
  public native void output(@Nullable String synthesizer, @Nullable String voice, @Nullable String language, @Nullable Byte rate, @Nullable Byte volume, @Nullable Byte pitch, @Nullable String brailleBackend, @NotNull String text, boolean interrupt);
}
