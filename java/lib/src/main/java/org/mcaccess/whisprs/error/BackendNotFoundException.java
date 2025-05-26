package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class BackendNotFoundException extends RuntimeException {
  public BackendNotFoundException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
