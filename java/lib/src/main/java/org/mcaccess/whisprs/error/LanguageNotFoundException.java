package org.mcaccess.whisprs.error;

import org.jetbrains.annotations.NotNull;

public class LanguageNotFoundException extends RuntimeException {
  public LanguageNotFoundException(@NotNull String errorMessage) {
    super(errorMessage);
  }
}
