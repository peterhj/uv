use crate::bindings::*;

#[repr(C)]
pub struct uv_req_t {

  // UV_REQ_FIELDS
  // public (req)
  pub data: *mut c_void,
  // read-only (req)
  pub type_: uv_req_type,
  // private (req)
  reserved: [*mut c_void; 6],
  // private (req, unix)
  // **empty**

}

#[repr(C)]
pub struct uv_shutdown_t {

  // UV_REQ_FIELDS
  // public (req)
  pub data: *mut c_void,
  // read-only (req)
  pub type_: uv_req_type,
  // private (req)
  reserved: [*mut c_void; 6],
  // private (req, unix)
  // **empty**

  pub handle: *mut uv_stream_t,
  pub cb: uv_shutdown_cb,

  // UV_SHUTDOWN_PRIVATE_FIELDS
  // **empty**

}

#[repr(C)]
pub struct uv_write_t {

  // UV_REQ_FIELDS
  // public (req)
  pub data: *mut c_void,
  // read-only (req)
  pub type_: uv_req_type,
  // private (req)
  reserved: [*mut c_void; 6],
  // private (req, unix)
  // **empty**

  pub cb: uv_write_cb,
  pub send_handle: *mut uv_stream_t,
  pub handle: *mut uv_stream_t,

  // UV_WRITE_PRIVATE_FIELDS
  queue: uv__queue,
  write_index: c_uint,
  bufs: *mut uv_buf_t,
  nbufs: c_uint,
  error: c_int,
  bufsml: [uv_buf_t; 4],

}

#[repr(C)]
pub struct uv_connect_t {

  // UV_REQ_FIELDS
  // public (req)
  pub data: *mut c_void,
  // read-only (req)
  pub type_: uv_req_type,
  // private (req)
  reserved: [*mut c_void; 6],
  // private (req, unix)
  // **empty**

  pub cb: uv_connect_cb,
  pub handle: *mut uv_stream_t,

  // UV_CONNECT_PRIVATE_FIELDS
  queue: uv__queue,

}

#[repr(C)]
pub struct uv_handle_t {

  // UV_HANDLE_FIELDS
  // public (handle)
  pub data: *mut c_void,
  // read-only (handle)
  pub loop_: *mut uv_loop_t,
  pub type_: uv_handle_type,
  // private (handle)
  close_cb: uv_close_cb,
  handle_queue: uv__queue,
  reserved: [*mut c_void; 4],
  // private (handle, unix)
  next_closing: *mut uv_handle_t,
  flags: c_uint,

}

#[repr(C)]
pub struct uv_stream_t {

  // UV_HANDLE_FIELDS
  // public (handle)
  pub data: *mut c_void,
  // read-only (handle)
  pub loop_: *mut uv_loop_t,
  pub type_: uv_handle_type,
  // private (handle)
  close_cb: uv_close_cb,
  handle_queue: uv__queue,
  reserved: [*mut c_void; 4],
  // private (handle, unix)
  next_closing: *mut uv_handle_t,
  flags: c_uint,


  // UV_STREAM_FIELDS
  // public (stream)
  pub write_queue_size: usize,
  pub alloc_cb: uv_alloc_cb,
  pub read_cb: uv_read_cb,
  // private (stream, unix)
  connect_req: *mut uv_connect_t,
  shutdown_req: *mut uv_shutdown_t,
  io_watcher: uv__io_t,
  write_queue: uv__queue,
  write_completed_queue: uv__queue,
  connection_cb: uv_connection_cb,
  delayed_error: c_int,
  accepted_fd: c_int,
  queued_fds: *mut c_void,

}

#[repr(C)]
pub struct uv_tcp_t {

  // UV_HANDLE_FIELDS
  // public (handle)
  pub data: *mut c_void,
  // read-only (handle)
  pub loop_: *mut uv_loop_t,
  pub type_: uv_handle_type,
  // private (handle)
  close_cb: uv_close_cb,
  handle_queue: uv__queue,
  reserved: [*mut c_void; 4],
  // private (handle, unix)
  next_closing: *mut uv_handle_t,
  flags: c_uint,


  // UV_STREAM_FIELDS
  // public (stream)
  pub write_queue_size: usize,
  pub alloc_cb: uv_alloc_cb,
  pub read_cb: uv_read_cb,
  // private (stream, unix)
  connect_req: *mut uv_connect_t,
  shutdown_req: *mut uv_shutdown_t,
  io_watcher: uv__io_t,
  write_queue: uv__queue,
  write_completed_queue: uv__queue,
  connection_cb: uv_connection_cb,
  delayed_error: c_int,
  accepted_fd: c_int,
  queued_fds: *mut c_void,


  // UV_TCP_PRIVATE_FIELDS
  // **empty**

}

#[repr(C)]
pub struct uv_async_t {

  // UV_HANDLE_FIELDS
  // public (handle)
  pub data: *mut c_void,
  // read-only (handle)
  pub loop_: *mut uv_loop_t,
  pub type_: uv_handle_type,
  // private (handle)
  close_cb: uv_close_cb,
  handle_queue: uv__queue,
  reserved: [*mut c_void; 4],
  // private (handle, unix)
  next_closing: *mut uv_handle_t,
  flags: c_uint,


  // UV_ASYNC_PRIVATE_FIELDS
  async_cb: uv_async_cb,
  queue: uv__queue,
  pending: c_int,

}

#[repr(C)]
pub struct uv_signal_t {
  pub signal_cb: uv_signal_cb,
  pub signum: c_int,

  // UV_SIGNAL_PRIVATE_FIELDS
  tree_entry: _rb_tree_entry,
  caught_signals: c_uint,
  dispatched_signals: c_uint,

}

#[repr(C)]
pub struct uv_loop_t {
  // User data
  pub data: *mut c_void,
  // Loop reference counting
  pub active_handles: c_uint,
  pub handle_queue: uv__queue,
  //pub active_reqs: _,
  pub unused: *mut c_void,
  // Internal
  pub internal_fields: *mut c_void,
  pub stop_flag: c_uint,

  // UV_LOOP_PRIVATE_FIELDS
  pub flags: usize,
  pub backend_fd: i32,
  pub pending_queue: uv__queue,
  pub watcher_queue: uv__queue,
  pub watchers: *mut *mut uv__io_t,
  pub nwatchers: u32,
  pub nfds: u32,
  pub wq: uv__queue,
  pub wq_mutex: uv_mutex_t,
  pub wq_async: uv_async_t,
  pub cloexec_lock: uv_rwlock_t,
  pub closing_handles: *mut uv_handle_t,
  pub process_handles: uv__queue,
  pub prepare_handles: uv__queue,
  pub check_handles: uv__queue,
  pub idle_handles: uv__queue,
  pub async_handles: uv__queue,
  pub async_unused: unsafe extern "C" fn(),
  pub async_io_watcher: uv__io_t,
  pub async_wfd: i32,
  pub timer_heap: _timer_heap,
  pub timer_counter: u64,
  pub time: u64,
  pub signal_pipefd: [i32; 2],
  pub signal_io_watcher: uv__io_t,
  pub child_watcher: uv_signal_t,
  pub emfile_fd: i32,

  // UV_PLATFORM_LOOP_FIELDS
  // **empty**


}

#[repr(C)]
pub struct _timer_heap {
  pub min: *mut c_void,
  pub nelts: c_uint,
}

#[repr(C)]
pub struct _rb_tree_entry {
  pub rbe_left: *mut uv_signal_t,
  pub rbe_right: *mut uv_signal_t,
  pub rbe_parent: *mut uv_signal_t,
  pub rbe_color: c_int,
}
