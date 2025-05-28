package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class InvalidParameterException extends RuntimeException {
  public InvalidParameterException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
