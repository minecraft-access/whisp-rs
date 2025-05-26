package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class InvalidPitchException extends RuntimeException {
  public InvalidPitchException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
