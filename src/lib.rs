extern crate libc;
#[cfg(feature = "addrsan")]
extern crate once_cell;
extern crate os_socketaddr;

#[cfg(feature = "addrsan")]
use crate::addrsan::{
  _uv_addrsan_malloc as malloc,
  _uv_addrsan_calloc as calloc,
  _uv_addrsan_realloc as realloc,
  _uv_addrsan_free as free,
};
use crate::bindings::*;

#[cfg(not(feature = "addrsan"))]
use libc::{malloc, calloc, realloc, free};

use os_socketaddr::{OsSocketAddr};

use std::mem::{size_of};
use std::net::{ToSocketAddrs};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{null_mut};
use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::sync::{Arc};

#[cfg(feature = "addrsan")]
pub mod addrsan;
pub mod bindings;

pub fn init_once_uv() {
  #[cfg(feature = "addrsan")]
  {
    crate::addrsan::init();
  }
  // TODO: also verify sizes of certain struct bindings.
}

pub trait UvAllocCb {
  fn callback_raw(handle: *mut uv_handle_t, suggested_size: usize, buf: *mut uv_buf_t) {
    let handle = UvHandle::from_raw(handle);
    let buf = unsafe { &mut *(buf as *mut UvBuf) };
    <Self as UvAllocCb>::callback(handle, suggested_size, buf)
  }

  fn callback(handle: UvHandle, suggested_size: usize, buf: &mut UvBuf);
}

// NB: in the original FFI, `uv_read_cb` assumes an invariant that the
// buffer memory can be freed inside the callback, even though the buffer is
// provided to the callback via a const pointer...
//
// But, suppose the buffer is immediately fed as input to `uv_write`; then,
// the buffer is still being used and cannot yet be freed. Specifically, copies
// of the write buffers are stored in the `uv_write_t` request itself, though
// the buffer memory itself is not copied; c.f. `uv_write2`.
//
// Except, the write request buffers are freed _before_ the write callback;
// c.f. `uv__write_callbacks`.
//
// So, in these bindings, we apply the following patches:
//
// 1. upgrade `uv_read_cb` argument from const to mut pointer
// 2. free `uv_write_t` buffers _after_ call to `uv_write_cb`

pub trait UvReadCb {
  fn callback_raw(stream: *mut uv_stream_t, nread: isize, buf: *mut uv_buf_t) {
    let stream = UvStream::from_raw(stream);
    let buf = unsafe { &mut *(buf as *mut UvBuf) };
    <Self as UvReadCb>::callback(stream, nread, buf)
  }

  fn callback(stream: UvStream, nread: isize, buf: &mut UvBuf);
}

pub trait UvWriteCb {
  fn callback_raw(req: *mut uv_write_t, status: c_int) {
    let req = UvWrite::from_raw(req);
    <Self as UvWriteCb>::callback(req, status)
  }

  fn callback(req: UvWrite, status: c_int);
}

pub trait UvConnectCb {
  fn callback_raw(req: *mut uv_connect_t, status: c_int);
}

pub trait UvShutdownCb {
  fn callback_raw(req: *mut uv_shutdown_t, status: c_int) {
    let req = UvShutdown::from_raw(req);
    <Self as UvShutdownCb>::callback(req, status)
  }

  fn callback(req: UvShutdown, status: c_int);
}

pub trait UvConnectionCb {
  fn callback_raw(server: *mut uv_stream_t, status: c_int);
}

pub trait UvCloseCb {
  fn callback_raw(handle: *mut uv_handle_t) {
    let handle = UvHandle::from_raw(handle);
    <Self as UvCloseCb>::callback(handle)
  }

  fn callback(req: UvHandle);
}

pub trait UvAsyncCb {
  fn callback_raw(handle: *mut uv_async_t) {
    let async_ = UvAsync::from_raw(handle);
    <Self as UvAsyncCb>::callback(async_)
  }

  fn callback(async_: UvAsync);
}

pub trait UvSignalCb {
  fn callback_raw(handle: *mut uv_signal_t, signum: c_int) {
    let signal = UvSignal::from_raw(handle);
    <Self as UvSignalCb>::callback(signal, signum)
  }

  fn callback(signal: UvSignal, signum: c_int);
}

unsafe extern "C" fn alloc_trampoline<Cb: UvAllocCb>(handle: *mut uv_handle_t, suggested_size: usize, buf: *mut uv_buf_t) {
  Cb::callback_raw(handle, suggested_size, buf)
}

unsafe extern "C" fn read_trampoline<Cb: UvReadCb>(stream: *mut uv_stream_t, nread: isize, buf: *mut uv_buf_t) {
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

unsafe extern "C" fn async_trampoline<Cb: UvAsyncCb>(handle: *mut uv_async_t) {
  Cb::callback_raw(handle)
}

unsafe extern "C" fn signal_trampoline<Cb: UvSignalCb>(handle: *mut uv_signal_t, signum: c_int) {
  Cb::callback_raw(handle, signum)
}

#[repr(transparent)]
pub struct UvBuf {
  pub inner: uv_buf_t,
}

impl UvBuf {
  pub fn from_raw_parts_unchecked(base: *mut c_char, len: usize) -> UvBuf {
    let inner = uv_buf_t{base, len};
    UvBuf{inner}
  }

  pub fn replace_raw_parts_unchecked(&mut self, base: *mut c_char, len: usize) -> UvBuf {
    let prev = UvBuf{inner: self.inner};
    self.inner.base = base;
    self.inner.len = len;
    prev
  }

  /*pub fn alloc(&mut self, size: usize) {
    unsafe {
      self.inner.base = libc::malloc(size) as *mut _;
      self.inner.len = size;
    }
  }*/

  pub fn len(&self) -> usize {
    self.inner.len
  }

  pub fn as_bytes(&self) -> Option<&[u8]> {
    if self.inner.base.is_null() {
      return None;
    }
    if self.inner.len <= 0 {
      return None;
    }
    Some(unsafe { from_raw_parts(self.inner.base as *mut u8, self.inner.len) })
  }

  pub fn take_raw_parts(&mut self) -> (*mut c_char, usize) {
    let inner = self.inner;
    self.inner.base = null_mut();
    self.inner.len = 0;
    (inner.base, inner.len)
  }
}

pub struct UvLoop {
  inner: *mut uv_loop_t,
  // FIXME: deprecate refct?
  refct: Option<Arc<()>>,
}

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
    let inner = unsafe { malloc(size_of::<uv_loop_t>()) } as *mut _;
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

  pub fn stop(&self) {
    unsafe { uv_stop(self.inner) };
  }
}

pub struct UvHandle {
  inner: *mut uv_handle_t,
}

impl UvHandle {
  pub fn from_raw(inner: *mut uv_handle_t) -> UvHandle {
    UvHandle{inner}
  }

  pub fn _free_unchecked(self) {
    unsafe { free(self.inner as *mut c_void); }
  }
}

pub struct UvStream {
  inner: *mut uv_stream_t,
}

impl UvStream {
  pub fn from_raw(inner: *mut uv_stream_t) -> UvStream {
    UvStream{inner}
  }

  pub fn into_handle(self) -> UvHandle {
    UvHandle{inner: self.inner as _}
  }

  pub fn accept(&self, client: &UvTcp) -> Result<(), c_int> {
    let result = unsafe { uv_accept(self.inner, client.inner as _) };
    if result == 0 {
      Ok(())
    } else {
      Err(result)
    }
  }

  pub fn write<Cb: UvWriteCb>(&self, mut req: UvWrite, write_buf: &UvBuf) {
    let result = unsafe { uv_write(
        req.inner,
        self.inner,
        &write_buf.inner as *const _,
        1,
        write_trampoline::<Cb>,
    ) };
    req._forget_unchecked();
    // TODO
    println!("DEBUG: UvStream::write: result = {:?}", result);
  }

  pub fn shutdown<Cb: UvShutdownCb>(&self, mut req: UvShutdown) {
    let result = unsafe { uv_shutdown(
        req.inner,
        self.inner,
        shutdown_trampoline::<Cb>,
    ) };
    req._forget_unchecked();
    // TODO
    println!("DEBUG: UvStream::shutdown: result = {:?}", result);
  }
}

pub struct UvTcp {
  inner: *mut uv_tcp_t,
}

impl UvTcp {
  pub fn new(loop_: &UvLoop) -> UvTcp {
    let inner = unsafe { malloc(size_of::<uv_tcp_t>()) } as *mut _;
    unsafe { uv_tcp_init(loop_.inner, inner); }
    UvTcp{inner}
  }

  pub fn into_handle(self) -> UvHandle {
    UvHandle{inner: self.inner as _}
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
        &write_buf.inner as *const _,
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

pub struct UvReq {
  inner: *mut uv_req_t,
}

impl UvReq {
  pub fn _free_unchecked(self) {
    unsafe { free(self.inner as *mut c_void); }
  }

  pub fn _forget_unchecked(&mut self) {
    self.inner = null_mut();
  }
}

pub struct UvWrite {
  inner: *mut uv_write_t,
}

impl UvWrite {
  pub fn new() -> UvWrite {
    let inner = unsafe { malloc(size_of::<uv_write_t>()) } as *mut _;
    UvWrite{inner}
  }

  pub fn from_raw(inner: *mut uv_write_t) -> UvWrite {
    UvWrite{inner}
  }

  pub fn into_req(self) -> UvReq {
    UvReq{inner: self.inner as _}
  }

  pub fn _forget_unchecked(&mut self) {
    self.inner = null_mut();
  }

  #[inline]
  pub fn _inner_bufs_ptr(&self) -> *mut uv_buf_t {
    assert!(!self.inner.is_null());
    unsafe {
      let inner: &uv_write_t = &*self.inner;
      inner.bufs
    }
  }

  #[inline]
  pub fn _inner_bufs_len(&self) -> usize {
    assert!(!self.inner.is_null());
    unsafe {
      let inner: &uv_write_t = &*self.inner;
      inner.nbufs as _
    }
  }

  pub fn _inner_mut_bufs_unchecked(&mut self) -> Option<&mut [UvBuf]> {
    let ptr = self._inner_bufs_ptr();
    let len = self._inner_bufs_len();
    if ptr.is_null() {
      return None;
    }
    if len <= 0 {
      return None;
    }
    Some(unsafe { from_raw_parts_mut(ptr as *mut _, len) })
  }
}

pub struct UvShutdown {
  inner: *mut uv_shutdown_t,
}

impl UvShutdown {
  pub fn new() -> UvShutdown {
    let inner = unsafe { malloc(size_of::<uv_shutdown_t>()) } as *mut _;
    UvShutdown{inner}
  }

  pub fn from_raw(inner: *mut uv_shutdown_t) -> UvShutdown {
    UvShutdown{inner}
  }

  pub fn into_req(self) -> UvReq {
    UvReq{inner: self.inner as _}
  }

  pub fn as_raw(&self) -> &uv_shutdown_t {
    unsafe { &*self.inner }
  }

  pub fn _forget_unchecked(&mut self) {
    self.inner = null_mut();
  }
}

pub struct UvAsync {
  inner: *mut uv_async_t,
}

unsafe impl Send for UvAsync {}
unsafe impl Sync for UvAsync {}

impl UvAsync {
  pub fn new<Cb: UvAsyncCb>(loop_: &UvLoop) -> UvAsync {
    let inner = unsafe { malloc(size_of::<uv_async_t>()) } as *mut _;
    let _result = unsafe { uv_async_init(loop_.inner, inner, async_trampoline::<Cb>) };
    UvAsync{inner}
  }

  pub fn from_raw(inner: *mut uv_async_t) -> UvAsync {
    UvAsync{inner}
  }

  pub fn into_handle(self) -> UvHandle {
    UvHandle{inner: self.inner as _}
  }

  pub fn send_(&self) /*-> Result<> */{
    let _result = unsafe { uv_async_send(self.inner) };
  }
}

pub struct UvSignal {
  inner: *mut uv_signal_t,
}

impl UvSignal {
  pub fn new(loop_: &UvLoop) -> UvSignal {
    let inner = unsafe { malloc(size_of::<uv_signal_t>()) } as *mut _;
    let result = unsafe { uv_signal_init(loop_.inner, inner) };
    println!("DEBUG: UvSignal::new: result = {:?}", result);
    UvSignal{inner}
  }

  pub fn from_raw(inner: *mut uv_signal_t) -> UvSignal {
    UvSignal{inner}
  }

  pub fn start<Cb: UvSignalCb>(&self, signum: c_int) /*-> Result<> */{
    let result = unsafe { uv_signal_start(self.inner, signal_trampoline::<Cb>, signum) };
    println!("DEBUG: UvSignal::start: result = {:?}", result);
  }

  pub fn stop(&self) /*-> Result<> */{
    let result = unsafe { uv_signal_stop(self.inner) };
    println!("DEBUG: UvSignal::stop: result = {:?}", result);
  }
}
