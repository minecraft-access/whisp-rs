package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class InvalidRateException extends RuntimeException {
  public InvalidRateException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
