package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class UnknownException extends RuntimeException {
  public UnknownException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
