#![allow(unused_variables)]

//extern crate once_cell;
extern crate uv;

//use once_cell::sync::{Lazy};
use uv::*;
use uv::bindings::*;

use std::cell::{RefCell};
use std::collections::{HashSet};
use std::os::raw::{c_int};

//static RESPONSE: &'static [u8] = b"Hello, world!\n";

thread_local! {
  static BACKEND: Backend = Backend::new();
}

#[derive(Default)]
pub struct Store {
  buf: HashSet<Box<[u8]>>,
}

pub struct Backend {
  loop_: UvLoop,
  sig: UvSignal,
  tcp: UvTcp,
  store: RefCell<Store>,
}

impl Backend {
  pub fn new() -> Backend {
    let loop_ = UvLoop::new();
    let sig = UvSignal::new(&loop_);
    let tcp = UvTcp::new(&loop_);
    Backend{
      loop_,
      sig,
      tcp,
      store: RefCell::new(Store::default()),
    }
  }

  pub fn run(&self) {
    println!("DEBUG: Backend::run: signal...");
    //self.sig.start::<Backend>(2);
    self.sig.start::<Backend>(15);
    println!("DEBUG: Backend::run: bind...");
    self.tcp.bind(("127.0.0.1", 8080));
    println!("DEBUG: Backend::run: listen...");
    self.tcp.listen::<Backend>();
    println!("DEBUG: Backend::run: run...");
    self.loop_.run();
  }

  pub fn stop(&self) {
    println!("DEBUG: Backend::run: stop...");
    self.loop_.stop();
  }
}

impl UvAllocCb for Backend {
  fn callback(_handle: UvHandle, suggested_size: usize, buf: &mut UvBuf) {
    println!("DEBUG: Backend: alloc callback: size = {} buf.len = {:?}", suggested_size, buf.as_bytes().map(|b| b.len()));
    BACKEND.with(|backend| {
      let mut store = backend.store.borrow_mut();
      let mut backing_buf: Vec<u8> = Vec::with_capacity(suggested_size);
      backing_buf.resize(suggested_size, 0);
      let mut backing_buf: Box<[u8]> = backing_buf.into();
      let _ = buf.replace_raw_parts_unchecked(backing_buf.as_mut_ptr() as _, backing_buf.len());
    });
  }
}

impl UvReadCb for Backend {
  fn callback(client: UvStream, nread: isize, buf: &UvBuf) {
    println!("DEBUG: Backend: read callback: nread = {} buf.len = {}", nread, buf.len());
    if nread < 0 {
      let errno = nread as c_int;
      if errno == UV_EOF {
        println!("DEBUG: Backend: read callback: eof");
      } else {
        // FIXME
        println!("DEBUG: Backend: read callback: error = {}", errno);
      }
      let req = UvShutdown::new();
      client.shutdown::<Backend>(req);
      return;
    }
    println!("DEBUG: Backend: read callback: write response...");
    let res_str = format!("Hello, world! {}\n", nread);
    // FIXME: assure buffer lifetime.
    let res_buf = res_str.as_bytes();
    let req = UvWrite::new();
    let buf = UvBuf::from_raw_parts_unchecked(res_buf.as_ptr() as _, res_buf.len());
    client.write::<Backend>(req, &buf);
    //let req = UvShutdown::new();
    //client.shutdown::<Backend>(req);
  }
}

impl UvWriteCb for Backend {
  fn callback(req: UvWrite, status: c_int) {
    println!("DEBUG: Backend: write callback: status = {}", status);
    req.into_req()._free_unchecked();
  }
}

impl UvShutdownCb for Backend {
  fn callback(req: UvShutdown, status: c_int) {
    println!("DEBUG: Backend: shutdown callback: status = {}", status);
    req.into_req()._free_unchecked();
  }
}

impl UvConnectionCb for Backend {
  fn callback_raw(server: *mut uv_stream_t, status: c_int) {
    println!("DEBUG: Backend: connection callback: hello... status = {}", status);
    BACKEND.with(|backend| {
      //let backend = &*BACKEND;
      let loop_ = &backend.loop_;
      let client = UvTcp::new(loop_);
      let stream = UvStream::from_raw(server);
      match stream.accept(&client) {
        Err(e) => {
          println!("DEBUG: Backend: connection callback: accept: err = {e}");
          let req = UvShutdown::new();
          client.shutdown::<Backend>(req);
        }
        Ok(_) => {
          println!("DEBUG: Backend: connection callback: accept: ok");
          client.read_start::<Backend, Backend>();
        }
      }
    });
  }
}

impl UvCloseCb for Backend {
  fn callback(handle: UvHandle) {
    println!("DEBUG: Backend: close callback");
    handle._free_unchecked();
  }
}

impl UvSignalCb for Backend {
  fn callback(signal: UvSignal, signum: c_int) {
    println!("DEBUG: Backend: signal callback: signum = {}", signum);
    if signum == 2 || signum == 15 {
      BACKEND.with(|backend| {
        backend.stop();
      });
    }
    //signal.stop();
  }
}

fn main() {
  init_once_uv();
  let version = UvLoop::version();
  println!("DEBUG: main: uv version = {:?}", version);
  BACKEND.with(|backend| {
    //let backend = &*BACKEND;
    backend.run();
  });
}
