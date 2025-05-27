package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class NoBackendsException extends RuntimeException {
  public NoBackendsException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
