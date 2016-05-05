use std::env;
use std::path::{PathBuf};

fn main() {
  let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let host_triple = env::var("HOST").unwrap();
  let mut artifacts_path = PathBuf::from(&manifest_dir);
  artifacts_path.push(&format!("artifacts.{}", host_triple));
  println!("cargo:rustc-link-search=native={}", artifacts_path.to_str().unwrap());
}
