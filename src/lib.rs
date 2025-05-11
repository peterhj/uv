extern crate libc;
extern crate os_socketaddr;

use self::bindings::*;

use os_socketaddr::{OsSocketAddr};

use std::mem::{size_of};
use std::net::{ToSocketAddrs};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null_mut};
use std::sync::{Arc};

pub mod bindings;

pub trait UvAllocCb {
  fn callback_raw(handle: *mut uv_handle_t, suggested_size: usize, buf: *mut uv_buf_t);
}

pub trait UvReadCb {
  fn callback_raw(stream: *mut uv_stream_t, nread: isize, buf: *const uv_buf_t);
}

pub trait UvWriteCb {
  fn callback_raw(req: *mut uv_write_t, status: c_int);
}

pub trait UvConnectCb {
  fn callback_raw(req: *mut uv_connect_t, status: c_int);
}

pub trait UvShutdownCb {
  fn callback_raw(req: *mut uv_shutdown_t, status: c_int);
}

pub trait UvConnectionCb {
  fn callback_raw(server: *mut uv_stream_t, status: c_int);
}

pub trait UvCloseCb {
  fn callback_raw(handle: *mut uv_handle_t);
}

unsafe extern "C" fn alloc_trampoline<Cb: UvAllocCb>(handle: *mut uv_handle_t, suggested_size: usize, buf: *mut uv_buf_t) {
  Cb::callback_raw(handle, suggested_size, buf)
}

unsafe extern "C" fn read_trampoline<Cb: UvReadCb>(stream: *mut uv_stream_t, nread: isize, buf: *const uv_buf_t) {
  Cb::callback_raw(stream, nread, buf)
}

unsafe extern "C" fn write_trampoline<Cb: UvWriteCb>(req: *mut uv_write_t, status: c_int) {
  Cb::callback_raw(req, status)
}

unsafe extern "C" fn shutdown_trampoline<Cb: UvShutdownCb>(req: *mut uv_shutdown_t, status: c_int) {
  Cb::callback_raw(req, status)
}

unsafe extern "C" fn connection_trampoline<Cb: UvConnectionCb>(server: *mut uv_stream_t, status: c_int) {
  Cb::callback_raw(server, status)
}

unsafe extern "C" fn close_trampoline<Cb: UvCloseCb>(handle: *mut uv_handle_t) {
  Cb::callback_raw(handle)
}

#[repr(transparent)]
pub struct UvBuf {
  // TODO: buffer ownership?
  inner: *mut uv_buf_t,
  //inner: uv_buf_t,
}

impl UvBuf {
  pub fn from_raw_parts(base: *mut c_char, len: usize) -> UvBuf {
    let inner = unsafe {
      let ptr = libc::malloc(size_of::<uv_buf_t>()) as *mut _;
      {
        let buf: &mut uv_buf_t = &mut *ptr;
        buf.base = base;
        buf.len = len;
      }
      ptr
    };
    assert!(!inner.is_null());
    UvBuf{inner}
  }
}

pub struct UvLoop {
  inner: *mut uv_loop_t,
  refct: Option<Arc<()>>,
}

unsafe impl Send for UvLoop {}
unsafe impl Sync for UvLoop {}

impl Drop for UvLoop {
  fn drop(&mut self) {
    if self.refct.is_none() {
      return;
    }
    // TODO
  }
}

impl UvLoop {
  pub fn version() -> (u8, u8, u8) {
    let x: u32 = unsafe { uv_version() };
    let buf = x.to_le_bytes();
    (buf[2], buf[1], buf[0])
  }

  pub fn _default() -> UvLoop {
    let inner = unsafe { uv_default_loop() };
    UvLoop{inner, refct: None}
  }

  pub fn new() -> UvLoop {
    let inner = unsafe { libc::malloc(size_of::<uv_loop_t>()) } as *mut _;
    let result = unsafe { uv_loop_init(inner) };
    println!("DEBUG: UvLoop::init: result = {:?}", result);
    assert_eq!(result, 0);
    UvLoop{inner, refct: Some(Arc::new(()))}
  }

  /*pub fn close(self) {
    let _ = unsafe { uv_loop_close(self.inner) };
    self.inner = null_mut();
  }*/

  pub fn run(&self) {
    let result = unsafe { uv_run(self.inner, UV_RUN_DEFAULT) };
    println!("DEBUG: UvLoop::run: result = {:?}", result);
  }
}

pub struct UvStreamRef {
  inner: *mut uv_stream_t,
}

impl UvStreamRef {
  pub fn from_raw(inner: *mut uv_stream_t) -> UvStreamRef {
    UvStreamRef{inner}
  }

  pub fn accept(&self, client: &UvTcp) -> Result<(), c_int> {
    let result = unsafe { uv_accept(self.inner, client.inner as _) };
    if result == 0 {
      Ok(())
    } else {
      Err(result)
    }
  }
}

pub struct UvTcp {
  inner: *mut uv_tcp_t,
}

impl UvTcp {
  pub fn new(loop_: &UvLoop) -> UvTcp {
    let inner = unsafe { libc::malloc(size_of::<uv_tcp_t>()) } as *mut _;
    unsafe { uv_tcp_init(loop_.inner, inner); }
    UvTcp{inner}
  }

  pub fn as_ptr(&self) -> *mut uv_tcp_t {
    self.inner
  }

  pub fn as_stream_ptr(&self) -> *mut uv_stream_t {
    self.inner as *mut uv_stream_t
  }

  pub fn bind<A: ToSocketAddrs>(&self, addrs: A) {
    for addr in addrs.to_socket_addrs().unwrap() {
      println!("DEBUG: UvLoop::bind: addr = {:?}", addr);
      let os_addr = OsSocketAddr::from(addr);
      let result = unsafe { uv_tcp_bind(
          self.inner,
          os_addr.as_ptr() as *const c_void,
          0,
      ) };
      // TODO
      println!("DEBUG: UvTcp::bind: result = {:?}", result);
      break;
    }
  }

  pub fn listen<Cb: UvConnectionCb>(&self) {
    let result = unsafe { uv_listen(
        self.as_stream_ptr(),
        128,
        connection_trampoline::<Cb>,
    ) };
    // TODO
    println!("DEBUG: UvTcp::listen: result = {:?}", result);
  }

  pub fn read_start<AllocCb: UvAllocCb, ReadCb: UvReadCb>(&self) {
    let result = unsafe { uv_read_start(
        self.as_stream_ptr(),
        alloc_trampoline::<AllocCb>,
        read_trampoline::<ReadCb>,
    ) };
    // TODO
    println!("DEBUG: UvTcp::read_start: result = {:?}", result);
  }

  pub fn write<Cb: UvWriteCb>(&self, mut req: UvWrite, write_buf: &UvBuf) {
    let result = unsafe { uv_write(
        req.inner,
        self.as_stream_ptr(),
        write_buf.inner as *const _,
        1,
        write_trampoline::<Cb>,
    ) };
    req._forget_unchecked();
    // TODO
    println!("DEBUG: UvTcp::write: result = {:?}", result);
  }

  pub fn shutdown<Cb: UvShutdownCb>(&self, mut req: UvShutdown) {
    let result = unsafe { uv_shutdown(
        req.inner,
        self.as_stream_ptr(),
        shutdown_trampoline::<Cb>,
    ) };
    req._forget_unchecked();
    // TODO
    println!("DEBUG: UvTcp::shutdown: result = {:?}", result);
  }

  pub fn close<Cb: UvCloseCb>(&self) {
    unsafe { uv_close(self.inner as _, close_trampoline::<Cb>) }
  }
}

pub struct UvWrite {
  inner: *mut uv_write_t,
}

impl UvWrite {
  pub fn new() -> UvWrite {
    let inner = unsafe { libc::malloc(size_of::<uv_write_t>()) } as *mut _;
    UvWrite{inner}
  }

  pub fn _forget_unchecked(&mut self) {
    self.inner = null_mut();
  }
}

pub struct UvShutdown {
  inner: *mut uv_shutdown_t,
}

impl UvShutdown {
  pub fn from_raw(inner: *mut uv_shutdown_t) -> UvShutdown {
    UvShutdown{inner}
  }

  pub fn as_raw(&self) -> &uv_shutdown_t {
    unsafe { &*self.inner }
  }

  pub fn new() -> UvShutdown {
    let inner = unsafe { libc::malloc(size_of::<uv_shutdown_t>()) } as *mut _;
    UvShutdown{inner}
  }

  pub fn _forget_unchecked(&mut self) {
    self.inner = null_mut();
  }
}
