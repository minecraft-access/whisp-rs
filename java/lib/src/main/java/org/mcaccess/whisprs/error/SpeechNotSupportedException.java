package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class SpeechNotSupportedException extends RuntimeException {
  public SpeechNotSupportedException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
