package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class SpeakFailedException extends RuntimeException {
  public SpeakFailedException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
