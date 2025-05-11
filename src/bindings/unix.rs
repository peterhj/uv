use libc::{pthread_mutex_t, pthread_rwlock_t};

pub mod struct_bindings;

pub type uv_mutex_t = pthread_mutex_t;
pub type uv_rwlock_t = pthread_rwlock_t;
