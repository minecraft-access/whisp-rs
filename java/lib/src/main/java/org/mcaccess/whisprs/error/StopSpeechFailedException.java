package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class StopSpeechFailedException extends RuntimeException {
  public StopSpeechFailedException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
