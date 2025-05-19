#![allow(unused_variables)]

extern crate libc;
//extern crate once_cell;
extern crate uv;

//use once_cell::sync::{Lazy};
use uv::*;
use uv::bindings::*;

use std::cell::{RefCell};
use std::cmp::{Ordering};
use std::collections::{BTreeSet, HashSet};
use std::ptr::{null_mut};
use std::os::raw::{c_int, c_void};

//static RESPONSE: &'static [u8] = b"Hello, world!\n";

thread_local! {
  static BACKEND: Backend = Backend::new();
}

#[repr(C)]
pub struct BackingBuf {
  pub ptr: *mut u8,
  pub len: usize,
}

impl BackingBuf {
  pub fn maybe_from_raw_parts_unchecked(ptr: *mut u8, len: usize) -> Option<BackingBuf> {
    if ptr.is_null() {
      return None;
    }
    if len <= 0 {
      return None;
    }
    Some(BackingBuf{ptr, len})
  }

  pub fn from_raw_parts_unchecked(ptr: *mut u8, len: usize) -> BackingBuf {
    assert!(!ptr.is_null());
    assert!(len > 0);
    BackingBuf{ptr, len}
  }

  pub fn new_uninit(len: usize) -> BackingBuf {
    assert!(len > 0);
    let ptr = unsafe { libc::malloc(len) as *mut u8 };
    assert!(!ptr.is_null());
    BackingBuf{ptr, len}
  }

  pub fn free_unchecked(&mut self) {
    if self.ptr.is_null() {
      return;
    }
    unsafe { libc::free(self.ptr as *mut c_void); }
    self.ptr = null_mut();
  }
}

impl PartialEq for BackingBuf {
  fn eq(&self, other: &BackingBuf) -> bool {
    let e = self.ptr == other.ptr;
    if e {
      assert_eq!(self.len, other.len);
    }
    e
  }
}

impl Eq for BackingBuf {}

impl PartialOrd for BackingBuf {
  fn partial_cmp(&self, other: &BackingBuf) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for BackingBuf {
  fn cmp(&self, other: &BackingBuf) -> Ordering {
    let ord = self.ptr.cmp(&other.ptr);
    match ord {
      Ordering::Equal => {
        assert_eq!(self.len, other.len);
      }
      _ => {}
    }
    ord
  }
}

#[derive(Default)]
pub struct Store {
  buf: BTreeSet<BackingBuf>,
  //buf: HashSet<BackingBuf>,
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
      /*
      let mut backing_buf: Vec<u8> = Vec::with_capacity(suggested_size);
      backing_buf.resize(suggested_size, 0);
      let mut backing_buf: Box<[u8]> = backing_buf.into();
      let _ = buf.replace_raw_parts_unchecked(backing_buf.as_mut_ptr() as _, backing_buf.len());
      */
      println!("DEBUG: Backend: alloc callback: alloc backing buf...");
      let backing_buf = BackingBuf::new_uninit(suggested_size);
      let _ = buf.replace_raw_parts_unchecked(backing_buf.ptr as _, backing_buf.len);
      if let Some(_) = store.buf.replace(backing_buf) {
        println!("DEBUG: Backend: alloc callback: warning: backing buf was already stored!");
      }
    });
  }
}

impl UvReadCb for Backend {
  fn callback(client: UvStream, nread: isize, buf: &mut UvBuf) {
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
  fn callback(mut req: UvWrite, status: c_int) {
    println!("DEBUG: Backend: write callback: status = {}", status);
    if let Some(bufs) = req._inner_mut_bufs_unchecked() {
      println!("DEBUG: Backend: write callback: found req bufs: bufs.len = {}", bufs.len());
      for buf in bufs.iter_mut() {
        let (backing_ptr, backing_len) = buf.take_raw_parts();
        if let Some(mut backing_buf) = BackingBuf::maybe_from_raw_parts_unchecked(backing_ptr as _, backing_len) {
          BACKEND.with(|backend| {
            let mut store = backend.store.borrow_mut();
            if !store.buf.remove(&backing_buf) {
              println!("DEBUG: Backend: write callback: warning: backing buf was NOT in store!");
            }
            println!("DEBUG: Backend: write callback: free backing buf...");
            backing_buf.free_unchecked();
          });
        }
      }
    } else {
      println!("DEBUG: Backend: write callback: warning: no req bufs found!");
    }
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
