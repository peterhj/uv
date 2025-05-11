fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  // TODO: only pre-build is currently supported.
  println!("cargo:rerun-if-changed=_build/include/uv.h");
  println!("cargo:rerun-if-changed=_build/include/uv/darwin.h");
  println!("cargo:rerun-if-changed=_build/include/uv/errno.h");
  println!("cargo:rerun-if-changed=_build/include/uv/threadpool.h");
  println!("cargo:rerun-if-changed=_build/include/uv/unix.h");
  println!("cargo:rerun-if-changed=_build/include/uv/version.h");
  println!("cargo:rerun-if-changed=_build/lib/libuv.a");
  println!("cargo:rustc-link-search=native=_build/lib");
  println!("cargo:rustc-link-lib=static=uv");
}
