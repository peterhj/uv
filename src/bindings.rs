#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_int, c_uint, c_void};

#[cfg(unix)] pub use self::unix::*;
#[cfg(unix)] pub use self::unix::struct_bindings::*;

#[cfg(unix)] pub mod unix;

pub type uv_errno_t = c_int;
pub const UV_EOF:           uv_errno_t = -4095;
pub const UV_ERRNO_MAX:     uv_errno_t = -4096;

pub type uv_run_mode = c_int;
pub const UV_RUN_DEFAULT:   uv_run_mode = 0;
pub const UV_RUN_ONCE:      uv_run_mode = 1;
pub const UV_RUN_NOWAIT:    uv_run_mode = 2;

pub type uv_handle_type = c_int;
pub const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub const UV_ASYNC:         uv_handle_type = 1;
pub const UV_CHECK:         uv_handle_type = 2;
pub const UV_FS_EVENT:      uv_handle_type = 3;
pub const UV_FS_POLL:       uv_handle_type = 4;
pub const UV_HANDLE:        uv_handle_type = 5;
pub const UV_IDLE:          uv_handle_type = 6;
pub const UV_NAMED_PIPE:    uv_handle_type = 7;
pub const UV_POLL:          uv_handle_type = 8;
pub const UV_PREPARE:       uv_handle_type = 9;
pub const UV_PROCESS:       uv_handle_type = 10;
pub const UV_STREAM:        uv_handle_type = 11;
pub const UV_TCP:           uv_handle_type = 12;
pub const UV_TIMER:         uv_handle_type = 13;
pub const UV_TTY:           uv_handle_type = 14;
pub const UV_UDP:           uv_handle_type = 15;
pub const UV_SIGNAL:        uv_handle_type = 16;
pub const UV_FILE:          uv_handle_type = 17;
pub const UV_HANDLE_TYPE_MAX: uv_handle_type = 18;

pub type uv_req_type = c_int;
pub const UV_UNKNOWN_REQ:   uv_req_type = 0;
pub const UV_REQ:           uv_req_type = 1;
pub const UV_CONNECT:       uv_req_type = 2;
pub const UV_WRITE:         uv_req_type = 3;
pub const UV_SHUTDOWN:      uv_req_type = 4;
pub const UV_UDP_SEND:      uv_req_type = 5;
pub const UV_FS:            uv_req_type = 6;
pub const UV_WORK:          uv_req_type = 7;
pub const UV_GETADDRINFO:   uv_req_type = 8;
pub const UV_GETNAMEINFO:   uv_req_type = 9;
pub const UV_RANDOM:        uv_req_type = 10;
pub const UV_REQ_TYPE_PRIVATE: uv_req_type = 11;
pub const UV_REQ_TYPE_MAX:  uv_req_type = 12;

#[repr(C)]
pub struct uv__queue {
  next: *mut uv__queue,
  prev: *mut uv__queue,
}

pub type uv__io_cb = unsafe extern "C" fn (loop_: *mut uv_loop_t, w: *mut uv__io_t, events: c_uint);

#[repr(C)]
pub struct uv__io_t {
  cb: uv__io_cb,
  pending_queue: uv__queue,
  watcher_queue: uv__queue,
  pevents: c_uint,
  events: c_uint,
  fd: c_int,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct uv_buf_t {
  pub base: *mut c_char,
  pub len: usize,
}

pub type uv_alloc_cb = unsafe extern "C" fn (handle: *mut uv_handle_t, suggested_size: usize, buf: *mut uv_buf_t);
pub type uv_read_cb = unsafe extern "C" fn (stream: *mut uv_stream_t, nread: isize, buf: *mut uv_buf_t);
pub type uv_write_cb = unsafe extern "C" fn (req: *mut uv_write_t, status: c_int);
pub type uv_connect_cb = unsafe extern "C" fn (req: *mut uv_connect_t, status: c_int);
pub type uv_shutdown_cb = unsafe extern "C" fn (req: *mut uv_shutdown_t, status: c_int);
pub type uv_connection_cb = unsafe extern "C" fn (server: *mut uv_stream_t, status: c_int);
pub type uv_close_cb = unsafe extern "C" fn (handle: *mut uv_handle_t);
pub type uv_async_cb = unsafe extern "C" fn (handle: *mut uv_async_t);
pub type uv_signal_cb = unsafe extern "C" fn (handle: *mut uv_signal_t, signum: c_int);

extern "C" { pub fn uv_version() -> c_uint; }
extern "C" { pub fn uv_version_string() -> *const c_char; }
extern "C" { pub fn uv_library_shutdown(); }

extern "C" { pub fn uv_default_loop() -> *mut uv_loop_t; }
extern "C" { pub fn uv_loop_init(loop_: *mut uv_loop_t) -> c_int; }
extern "C" { pub fn uv_loop_close(loop_: *mut uv_loop_t) -> c_int; }
extern "C" { pub fn uv_run(loop_: *mut uv_loop_t, mode: uv_run_mode) -> c_int; }
extern "C" { pub fn uv_stop(loop_: *mut uv_loop_t); }

extern "C" { pub fn uv_ref(handle: *mut uv_handle_t); }
extern "C" { pub fn uv_unref(handle: *mut uv_handle_t); }
extern "C" { pub fn uv_has_ref(handle: *const uv_handle_t) -> c_int; }

extern "C" { pub fn uv_close(handle: *mut uv_handle_t, close_cb: uv_close_cb); }

extern "C" { pub fn uv_listen(stream: *mut uv_stream_t, backlog: c_int, cb: uv_connection_cb) -> c_int; }
extern "C" { pub fn uv_accept(server: *mut uv_stream_t, client: *mut uv_stream_t) -> c_int; }
extern "C" { pub fn uv_read_start(stream: *mut uv_stream_t, alloc_cb: uv_alloc_cb, read_cb: uv_read_cb) -> c_int; }
extern "C" { pub fn uv_write(req: *mut uv_write_t, handle: *mut uv_stream_t, bufs: *const uv_buf_t, nbufs: c_uint, cb: uv_write_cb) -> c_int; }
    
extern "C" { pub fn uv_tcp_init(loop_: *mut uv_loop_t, handle: *mut uv_tcp_t) -> c_int; }
extern "C" { pub fn uv_tcp_bind(handle: *mut uv_tcp_t, addr: *const c_void, flags: c_uint) -> c_int; }

extern "C" { pub fn uv_async_init(loop_: *mut uv_loop_t, async_: *mut uv_async_t, async_cb: uv_async_cb) -> c_int; }
extern "C" { pub fn uv_async_send(async_: *mut uv_async_t) -> c_int; }

extern "C" { pub fn uv_signal_init(loop_: *mut uv_loop_t, handle: *mut uv_signal_t) -> c_int; }
extern "C" { pub fn uv_signal_start(handle: *mut uv_signal_t, signal_cb: uv_signal_cb, signum: c_int) -> c_int; }
extern "C" { pub fn uv_signal_stop(handle: *mut uv_signal_t) -> c_int; }

extern "C" { pub fn uv_shutdown(req: *mut uv_shutdown_t, handle: *mut uv_stream_t, cb: uv_shutdown_cb) -> c_int; }
extern "C" { pub fn uv_ip4_addr(ip: *const c_char, port: c_int, addr: *mut c_void) -> c_int; }

pub type uv_malloc_func = unsafe extern "C" fn (usize) -> *mut c_void;
pub type uv_realloc_func = unsafe extern "C" fn (*mut c_void, usize) -> *mut c_void;
pub type uv_calloc_func = unsafe extern "C" fn (usize, usize) -> *mut c_void;
pub type uv_free_func = unsafe extern "C" fn (*mut c_void);

extern "C" { pub fn uv_replace_allocator(malloc_func: uv_malloc_func, realloc_func: uv_realloc_func, calloc_func: uv_calloc_func, free_func: uv_free_func) -> c_int; }
