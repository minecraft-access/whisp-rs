package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class AudioDataNotSupportedException extends RuntimeException {
  public AudioDataNotSupportedException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
