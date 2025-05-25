use std::path::{PathBuf};

fn main() {
  let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  println!("cargo:rerun-if-changed={}/build.rs", manifest_dir.display());
  // TODO: only pre-build is currently supported.
  println!("cargo:rerun-if-changed={}/_build/include/uv.h", manifest_dir.display());
  println!("cargo:rerun-if-changed={}/_build/include/uv/darwin.h", manifest_dir.display());
  println!("cargo:rerun-if-changed={}/_build/include/uv/errno.h", manifest_dir.display());
  println!("cargo:rerun-if-changed={}/_build/include/uv/threadpool.h", manifest_dir.display());
  println!("cargo:rerun-if-changed={}/_build/include/uv/unix.h", manifest_dir.display());
  println!("cargo:rerun-if-changed={}/_build/include/uv/version.h", manifest_dir.display());
  println!("cargo:rerun-if-changed={}/_build/lib/libuv.a", manifest_dir.display());
  println!("cargo:rustc-link-search=native={}/_build/lib", manifest_dir.display());
  println!("cargo:rustc-link-lib=static=uv");
}
