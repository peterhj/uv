#[cfg(feature = "addrsan")]
use crate::addrsan::{
  _uv_addrsan_malloc as malloc,
  _uv_addrsan_free as free,
};

#[cfg(not(feature = "addrsan"))]
use libc::{malloc, free};

use std::cmp::{Ordering};
use std::os::raw::{c_void};
use std::hash::{Hash, Hasher};
use std::ptr::{null_mut};
use std::slice::{from_raw_parts, from_raw_parts_mut};

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
    let ptr = unsafe { malloc(len) as *mut u8 };
    assert!(!ptr.is_null());
    BackingBuf{ptr, len}
  }

  pub fn free_unchecked(&mut self) {
    if self.ptr.is_null() {
      return;
    }
    unsafe { free(self.ptr as *mut c_void); }
    self.ptr = null_mut();
  }

  pub fn as_ptr(&self) -> *mut u8 {
    self.ptr
  }

  pub fn len(&self) -> usize {
    self.len
  }

  pub fn as_bytes(&self) -> &[u8] {
    unsafe { from_raw_parts(self.ptr, self.len) }
  }

  pub fn as_mut_bytes(&mut self) -> &mut [u8] {
    unsafe { from_raw_parts_mut(self.ptr, self.len) }
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

impl Hash for BackingBuf {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.ptr.hash(state);
  }
}
