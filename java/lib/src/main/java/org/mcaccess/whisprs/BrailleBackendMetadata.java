package org.mcaccess.whisprs;

import org.jetbrains.annotations.NotNull;

public class BrailleBackendMetadata {
  public final @NotNull String name;
  public final byte priority;
  public BrailleBackendMetadata(@NotNull String name, byte priority) {
    this.name = name;
    this.priority = priority;
  }
}
