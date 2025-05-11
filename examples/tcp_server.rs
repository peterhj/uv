#![allow(unused_variables)]

extern crate once_cell;
extern crate uv;

use once_cell::sync::{Lazy};
use uv::*;
use uv::bindings::*;

use std::os::raw::{c_int};

static RESPONSE: &'static [u8] = b"Hello, world!\n";

static BACKEND: Lazy<Backend> = Lazy::new(|| Backend::new());

pub struct Backend {
  loop_: UvLoop,
}

impl Backend {
  pub fn new() -> Backend {
    Backend{
      loop_: UvLoop::new(),
    }
  }
}

pub struct Server;

impl UvWriteCb for Server {
  fn callback_raw(req: *mut uv_write_t, status: c_int) {
  }
}

impl UvShutdownCb for Server {
  fn callback_raw(req: *mut uv_shutdown_t, status: c_int) {
  }
}

impl UvConnectionCb for Server {
  fn callback_raw(server: *mut uv_stream_t, status: c_int) {
    println!("DEBUG: Server::callback: hello...");
    let backend = &*BACKEND;
    let loop_ = &backend.loop_;
    let client = UvTcp::new(loop_);
    let stream = UvStreamRef::from_raw(server);
    match stream.accept(&client) {
      Err(e) => {
        println!("DEBUG: Server::callback: accept: err = {e}");
        client.close::<Server>();
      }
      Ok(_) => {
        println!("DEBUG: Server::callback: accept: ok");
        let req = UvWrite::new();
        let buf = UvBuf::from_raw_parts(RESPONSE.as_ptr() as _, RESPONSE.len());
        client.write::<Server>(req, &buf);
        let req = UvShutdown::new();
        client.shutdown::<Server>(req);
      }
    }
  }
}

impl UvCloseCb for Server {
  fn callback_raw(handle: *mut uv_handle_t) {
  }
}

fn main() {
  let version = UvLoop::version();
  println!("DEBUG: main: uv version = {:?}", version);
  let backend = &*BACKEND;
  let loop_ = &backend.loop_;
  let tcp = UvTcp::new(loop_);
  println!("DEBUG: main: bind...");
  tcp.bind(("127.0.0.1", 8080));
  println!("DEBUG: main: listen...");
  tcp.listen::<Server>();
  println!("DEBUG: main: run...");
  loop_.run();
}
