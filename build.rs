use build_target::Arch;
use build_target::Os;
use cc::Build;
use std::env;
use std::path::Path;
use std::process::Command;
fn main() {
  if Os::target().unwrap()==Os::Windows {
    let output_dir = env::var("OUT_DIR").unwrap();
    Command::new("midl")
      .arg("/server")
      .arg("none")
      .arg("/prefix")
      .arg("all")
      .arg("nvdaController_")
      .arg(match Arch::target().unwrap() {
        Arch::X86_64 => "/x64",
        Arch::AARCH64 => "/arm64",
        _ => panic!("Unsupported CPU archetecture")
      })
      .arg("/out")
      .arg(&format!("{}", output_dir))
      .arg("/acf")
      .arg("nvda_controller\\nvdaController.acf")
      .arg("nvda_controller\\nvdaController.idl")
      .status()
      .unwrap();
    Build::new()
      .file(Path::new(&output_dir).join("nvdaController_c.c"))
      .file(Path::new("nvda_controller").join("winIPCUtils.cpp"))
      .file(Path::new("nvda_controller").join("client.cpp"))
      .cpp(true)
      .compile("nvda_controller");
    println!("cargo::rustc-link-search=native={}", output_dir);
    println!("cargo::rustc-link-lib=static=nvda_controller");
    println!("cargo::rustc-link-lib=rpcrt4");
    let nvda_bindings = bindgen::Builder::default()
      .header(Path::new("nvda_controller").join("nvdaController.h").display().to_string())
      .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
      .allowlist_function("nvdaController_.+")
      .prepend_enum_name(false)
      .must_use_type("error_status_t")
      .generate()
      .unwrap();
    nvda_bindings
          .write_to_file(Path::new(&output_dir).join("nvda_bindings.rs"))
      .unwrap();
  }
}
