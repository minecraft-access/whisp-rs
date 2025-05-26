package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class BrailleFailedException extends RuntimeException {
  public BrailleFailedException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
