use crate::bindings::*;

use libc::{malloc, free};
use once_cell::sync::{Lazy};
use parking_lot::{Mutex};

use std::cmp::{min};
use std::collections::{HashMap};
use std::os::raw::{c_void};
use std::sync::{Arc};

static ADDRSAN_STATE: Lazy<Arc<Mutex<_AddrsanState>>> = Lazy::new(|| Arc::new(Mutex::new(_AddrsanState::default())));

#[derive(Default)]
struct _AddrsanState {
  ctr: u64,
  cache: HashMap<u64, (usize, usize)>,
  index: HashMap<usize, u64>,
}

impl _AddrsanState {
  pub fn fresh(&mut self) -> u64 {
    let next_ctr = self.ctr + 1;
    self.ctr = next_ctr;
    next_ctr
  }
}

pub fn init() {
  let result = unsafe {
    uv_replace_allocator(
        _uv_addrsan_malloc,
        _uv_addrsan_realloc,
        _uv_addrsan_calloc,
        _uv_addrsan_free,
    )
  };
  assert_eq!(result, 0);
}

pub unsafe extern "C" fn _uv_addrsan_malloc(size: usize) -> *mut c_void {
  // FIXME: assumes 64-bit min alignment.
  let ext_size = (((size + 8 - 1) / 8) + 1) * 8;
  let ptr = malloc(ext_size);
  let ptr_val = ptr as usize;
  let ptr_buf = ptr as *mut u8;
  {
    let mut state = ADDRSAN_STATE.lock();
    let ctr = state.fresh();
    assert!(state.cache.insert(ctr, (ptr_val, size)).is_none());
    assert!(state.index.insert(ptr_val, ctr).is_none());
    let ext_ptr = ptr_buf.offset((ext_size - 8) as _);
    assert_eq!((ext_ptr as usize) % 8, 0);
    std::ptr::write((ext_ptr as usize) as *mut u64, ctr);
  }
  ptr
}

pub unsafe extern "C" fn _uv_addrsan_calloc(nelem: usize, elemsz: usize) -> *mut c_void {
  // NB: require overflow-checks.
  let size = nelem * elemsz;
  // FIXME: assumes 64-bit min alignment.
  let ext_size = (((size + 8 - 1) / 8) + 1) * 8;
  let ptr = malloc(ext_size);
  let ptr_val = ptr as usize;
  let ptr_buf = ptr as *mut u8;
  for off in 0 .. (size as isize) {
    std::ptr::write(ptr_buf.offset(off), 0);
  }
  {
    let mut state = ADDRSAN_STATE.lock();
    let ctr = state.fresh();
    assert!(state.cache.insert(ctr, (ptr_val, size)).is_none());
    assert!(state.index.insert(ptr_val, ctr).is_none());
    let ext_ptr = ptr_buf.offset((ext_size - 8) as _);
    assert_eq!((ext_ptr as usize) % 8, 0);
    std::ptr::write((ext_ptr as usize) as *mut u64, ctr);
  }
  ptr
}

pub unsafe extern "C" fn _uv_addrsan_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
  let new_ptr = _uv_addrsan_malloc(size);
  if ptr.is_null() {
    return new_ptr;
  }
  let new_ptr_buf = new_ptr as *mut u8;
  let ptr_val = ptr as usize;
  let ptr_buf = ptr as *mut u8;
  {
    let mut state = ADDRSAN_STATE.lock();
    let index_old_ctr = match state.index.remove(&ptr_val) {
      None => {
        panic!("bug: ptr = 0x{:016x}", ptr_val);
      }
      Some(ctr) => ctr
    };
    match state.cache.remove(&index_old_ctr) {
      None => {
        panic!("bug");
      }
      Some((cache_ptr_val, cache_size)) => {
        assert_eq!(ptr_val, cache_ptr_val);
        let cache_ext_size = (((cache_size + 8 - 1) / 8) + 1) * 8;
        let ext_ptr = ptr_buf.offset((cache_ext_size - 8) as _);
        assert_eq!((ext_ptr as usize) % 8, 0);
        let ctr = std::ptr::read((ext_ptr as usize) as *mut u64);
        assert_eq!(ctr, index_old_ctr);
        // TODO
        std::ptr::copy_nonoverlapping(
            ptr_buf as *const _,
            new_ptr_buf,
            min(cache_size, size)
        );
      }
    }
  }
  free(ptr);
  new_ptr
}

pub unsafe extern "C" fn _uv_addrsan_free(ptr: *mut c_void) {
  if ptr.is_null() {
    println!("_uv_addrsan_free: warning: null ptr");
    return;
  }
  let ptr_val = ptr as usize;
  let ptr_buf = ptr as *mut u8;
  {
    let mut state = ADDRSAN_STATE.lock();
    let index_ctr = match state.index.remove(&ptr_val) {
      None => {
        panic!("_uv_addrsan_free: bug: double free: ptr = 0x{:016x}", ptr_val);
      }
      Some(ctr) => ctr
    };
    match state.cache.remove(&index_ctr) {
      None => {
        panic!("bug");
      }
      Some((cache_ptr_val, cache_size)) => {
        assert_eq!(ptr_val, cache_ptr_val);
        let cache_ext_size = (((cache_size + 8 - 1) / 8) + 1) * 8;
        let ext_ptr = ptr_buf.offset((cache_ext_size - 8) as _);
        assert_eq!((ext_ptr as usize) % 8, 0);
        let ctr = std::ptr::read((ext_ptr as usize) as *mut u64);
        assert_eq!(ctr, index_ctr);
      }
    }
  }
  free(ptr);
}
