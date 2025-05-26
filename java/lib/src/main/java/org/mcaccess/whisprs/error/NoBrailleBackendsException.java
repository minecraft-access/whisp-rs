package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class NoBrailleBackendsException extends RuntimeException {
  public NoBrailleBackendsException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
