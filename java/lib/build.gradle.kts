plugins {
    `java-library`
}
repositories {
    mavenCentral()
}
dependencies {
  compileOnly("org.jetbrains:annotations:26.0.2")
}
java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
}
