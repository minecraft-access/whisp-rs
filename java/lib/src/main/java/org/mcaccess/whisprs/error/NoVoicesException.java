package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class NoVoicesException extends RuntimeException {
  public NoVoicesException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
