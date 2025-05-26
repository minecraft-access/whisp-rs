package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class InvalidVolumeException extends RuntimeException {
  public InvalidVolumeException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
