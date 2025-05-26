package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class VoiceNotFoundException extends RuntimeException {
  public VoiceNotFoundException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
