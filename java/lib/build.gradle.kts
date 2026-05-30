plugins {
  `java-library`
}
group = "org.mcaccess"
version = "0.4.0"
repositories {
  mavenCentral()
}
dependencies {
  compileOnly("org.jetbrains:annotations:26.1.0")
}
java {
  toolchain {
    languageVersion = JavaLanguageVersion.of(25)
  }
}
val repoRoot = rootProject.projectDir.parentFile
val generatedNatives = layout.buildDirectory.dir("generated/natives")
val hostOs = System.getProperty("os.name").lowercase().let { name ->
  when {
    name.contains("win") -> "windows"
    name.contains("mac") || name.contains("darwin") -> "macos"
    name.contains("linux") -> "linux"
    else -> throw GradleException("Unsupported operating system: $name")
  }
}
val hostArch = when (val arch = System.getProperty("os.arch").lowercase()) {
  "amd64", "x86_64", "x64" -> "x86_64"
  "aarch64", "arm64" -> "aarch64"
  else -> throw GradleException("Unsupported CPU architecture: $arch")
}
val hostLibFile = when (hostOs) {
  "windows" -> "whisp_rs.dll"
  "macos" -> "libwhisp_rs.dylib"
  else -> "libwhisp_rs.so"
}
val cargoBuildHost by tasks.registering(Exec::class) {
  workingDir = repoRoot
  commandLine("cargo", "build", "--release")
}
val prepareLocalNatives by tasks.registering(Copy::class) {
  dependsOn(cargoBuildHost)
  into(generatedNatives)
  from(repoRoot.resolve("target/release/$hostLibFile")) {
    into("natives/$hostOs-$hostArch")
  }
  from(repoRoot.resolve("espeak-ng-data.zip"))
}
sourceSets.main {
  resources.srcDir(generatedNatives)
}
tasks.named<ProcessResources>("processResources") {
  mustRunAfter(prepareLocalNatives)
  duplicatesStrategy = DuplicatesStrategy.INCLUDE
}
