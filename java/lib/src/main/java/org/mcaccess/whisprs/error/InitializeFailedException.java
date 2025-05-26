package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class InitializeFailedException extends RuntimeException {
  public InitializeFailedException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
