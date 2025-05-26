package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class BrailleNotSupportedException extends RuntimeException {
  public BrailleNotSupportedException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
