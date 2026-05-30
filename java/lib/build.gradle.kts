plugins {
  `java-library`
}
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
