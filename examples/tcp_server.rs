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

impl UvAllocCb for Server {
  fn callback(_handle: UvHandle, suggested_size: usize, buf: &mut UvBuf) {
    buf.alloc(suggested_size);
  }
}

impl UvReadCb for Server {
  fn callback(client: UvStream, nread: isize, buf: &UvBuf) {
    println!("DEBUG: Server: read callback: nread = {} buf.len = {}", nread, buf.len());
    if nread < 0 {
      let errno = nread as c_int;
      if errno == UV_EOF {
        println!("DEBUG: Server: read callback: eof");
      } else {
        // FIXME
        println!("DEBUG: Server: read callback: error = {}", errno);
      }
      let req = UvShutdown::new();
      client.shutdown::<Server>(req);
      return;
    }
    println!("DEBUG: Server: read callback: write response...");
    let req = UvWrite::new();
    let buf = UvBuf::from_raw_parts_unchecked(RESPONSE.as_ptr() as _, RESPONSE.len());
    client.write::<Server>(req, &buf);
    //let req = UvShutdown::new();
    //client.shutdown::<Server>(req);
  }
}

impl UvWriteCb for Server {
  fn callback_raw(req: *mut uv_write_t, status: c_int) {
    println!("DEBUG: Server: write callback: status = {}", status);
  }
}

impl UvShutdownCb for Server {
  fn callback_raw(req: *mut uv_shutdown_t, status: c_int) {
    println!("DEBUG: Server: shutdown callback: status = {}", status);
  }
}

impl UvConnectionCb for Server {
  fn callback_raw(server: *mut uv_stream_t, status: c_int) {
    println!("DEBUG: Server: connection callback: hello... status = {}", status);
    let backend = &*BACKEND;
    let loop_ = &backend.loop_;
    let client = UvTcp::new(loop_);
    let stream = UvStream::from_raw(server);
    match stream.accept(&client) {
      Err(e) => {
        println!("DEBUG: Server: connection callback: accept: err = {e}");
        client.close::<Server>();
      }
      Ok(_) => {
        println!("DEBUG: Server: connection callback: accept: ok");
        client.read_start::<Server, Server>();
      }
    }
  }
}

impl UvCloseCb for Server {
  fn callback_raw(handle: *mut uv_handle_t) {
  }
}

fn main() {
  init_uv();
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
