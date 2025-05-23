#[cfg(target_os = "macos")]
pub mod av_speech_synthesizer;
pub mod espeak_ng;
#[cfg(windows)]
pub mod jaws;
#[cfg(windows)]
pub mod nvda;
#[cfg(windows)]
pub mod one_core;
#[cfg(windows)]
pub mod sapi;
#[cfg(target_os = "linux")]
pub mod speech_dispatcher;
