use build_target;
use build_target::Os;

fn main() {
  let os = build_target::target_os().unwrap();
  if os==Os::MacOs {
    println!("cargo:rustc-link-lib=c++");
  }
}
