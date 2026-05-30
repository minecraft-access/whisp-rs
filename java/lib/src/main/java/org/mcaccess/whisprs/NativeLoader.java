package org.mcaccess.whisprs;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.StandardCopyOption;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.Locale;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

final class NativeLoader {
  private static boolean loaded = false;
  private NativeLoader() {}
  static synchronized void load() {
    if (loaded) {
      return;
    }
    String resourceDir = osName() + "-" + osArch();
    Path libPath = extractLibrary("/natives/" + resourceDir + "/" + libFileName());
    Path dataParent = extractEspeakData();
    System.load(libPath.toAbsolutePath().toString());
    Whisprs.setEspeakDataPath(dataParent.toAbsolutePath().toString());
    loaded = true;
  }
  private static String osName() {
    String name = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
    if (name.contains("win")) {
      return "windows";
    }
    if (name.contains("mac") || name.contains("darwin")) {
      return "macos";
    }
    if (name.contains("linux")) {
      return "linux";
    }
    throw new UnsupportedOperationException("Unsupported operating system: " + name);
  }
  private static String osArch() {
    String arch = System.getProperty("os.arch", "").toLowerCase(Locale.ROOT);
    return switch (arch) {
      case "amd64", "x86_64", "x64" -> "x86_64";
      case "aarch64", "arm64" -> "aarch64";
      default -> throw new UnsupportedOperationException("Unsupported CPU architecture: " + arch);
    };
  }
  private static String libFileName() {
    return switch (osName()) {
      case "windows" -> "whisp_rs.dll";
      case "macos" -> "libwhisp_rs.dylib";
      default -> "libwhisp_rs.so";
    };
  }
  private static Path extractLibrary(String resource) {
    String fileName = resource.substring(resource.lastIndexOf('/') + 1);
    try (InputStream in = NativeLoader.class.getResourceAsStream(resource)) {
      if (in == null) {
        throw new UnsupportedOperationException("Native library not bundled in JAR: " + resource);
      }
      Path dir = createTempDirectory("whisprs-native");
      Path target = dir.resolve(fileName);
      Files.copy(in, target, StandardCopyOption.REPLACE_EXISTING);
      return target;
    } catch (IOException e) {
      throw new UncheckedIOException("Failed to extract native library " + resource, e);
    }
  }
  private static Path extractEspeakData() {
    String resource = "/espeak-ng-data.zip";
    try (InputStream in = NativeLoader.class.getResourceAsStream(resource)) {
      if (in == null) {
        throw new UnsupportedOperationException("espeak-ng-data.zip not bundled in JAR");
      }
      Path dir = createTempDirectory("whisprs-espeak");
      try (ZipInputStream zip = new ZipInputStream(in)) {
        ZipEntry entry;
        while ((entry = zip.getNextEntry()) != null) {
          Path resolved = dir.resolve(entry.getName()).normalize();
          if (!resolved.startsWith(dir)) {
            throw new IOException("Refusing to extract entry outside target directory: "
                + entry.getName());
          }
          if (entry.isDirectory()) {
            Files.createDirectories(resolved);
          } else {
            Files.createDirectories(resolved.getParent());
            Files.copy(zip, resolved, StandardCopyOption.REPLACE_EXISTING);
          }
        }
      }
      return dir;
    } catch (IOException e) {
      throw new UncheckedIOException("Failed to extract espeak-ng-data", e);
    }
  }
  private static Path createTempDirectory(String prefix) throws IOException {
    Path dir = Files.createTempDirectory(prefix);
    Runtime.getRuntime().addShutdownHook(new Thread(() -> deleteRecursively(dir)));
    return dir;
  }
  private static void deleteRecursively(Path dir) {
    try {
      if (!Files.exists(dir)) {
        return;
      }
      Files.walkFileTree(dir, new SimpleFileVisitor<Path>() {
        @Override
        public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) throws IOException {
          Files.deleteIfExists(file);
          return FileVisitResult.CONTINUE;
        }
        @Override
        public FileVisitResult postVisitDirectory(Path d, IOException exc) throws IOException {
          Files.deleteIfExists(d);
          return FileVisitResult.CONTINUE;
        }
      });
    } catch (IOException ignored) {
    }
  }
}
