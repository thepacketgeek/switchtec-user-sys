#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::ffi::{CStr, CString};
use std::fmt;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct SwitchtecDevice<T: AsRef<Path>> {
    path_ref: T,
}

impl<T> SwitchtecDevice<T>
where
    T: AsRef<Path>,
{
    pub fn new(path_ref: T) -> Self {
        Self { path_ref }
    }

    pub fn open(&self) -> io::Result<SwitchtecDeviceGuard> {
        SwitchtecDeviceGuard::new(self.path_ref.as_ref())
    }
}

impl<T> fmt::Debug for SwitchtecDevice<T>
where
    T: AsRef<Path>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SwitchtecDevice")
            .field("path", &self.path_ref.as_ref())
            .finish()
    }
}

pub struct SwitchtecDeviceGuard {
    pub inner: *mut switchtec_dev,
}

impl SwitchtecDeviceGuard {
    fn new<T: AsRef<Path>>(path: T) -> io::Result<Self> {
        let path_c = CString::new(path.as_ref().as_os_str().as_bytes()).map_err(|e| {
            // TODO: change to io::ErrorKind::InvalidFilename when it stabalizes
            //       https://github.com/rust-lang/rust/issues/86442
            io::Error::new(io::ErrorKind::Other, e.to_string())
        })?;
        unsafe {
            let dev = switchtec_open(path_c.as_ptr());
            if dev.is_null() {
                let err = CStr::from_ptr(switchtec_strerror()).to_owned();
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    err.into_string()
                        .unwrap_or_else(|_| "Unknown error".to_owned()),
                ))
            } else {
                Ok(Self { inner: dev })
            }
        }
    }
}

impl std::ops::Deref for SwitchtecDeviceGuard {
    type Target = *mut switchtec_dev;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::Drop for SwitchtecDeviceGuard {
    fn drop(&mut self) {
        unsafe {
            switchtec_close(self.inner);
        }
    }
}

impl switchtec_dev {
    pub fn new() -> Self {
        Self { _unused: [0; 0] }
    }
}
