#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{CStr, CString};
use std::fmt;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// `SwitchtecDevice` offers an safer way to work with the underlying [`switchtec_dev`]
///
/// - Provides an `open()` method that can only be called with a `switchtec_dev` that is non-null
/// - [`SwitchtecDeviceGuard`] closes the Switchtec character device when it goes out of scope
pub struct SwitchtecDevice<T: AsRef<Path>> {
    path_ref: T,
}

impl<T> SwitchtecDevice<T>
where
    T: AsRef<Path>,
{
    /// Construct a `SwitchtecDevice` for the given path
    ///
    /// ```
    /// use switchtec_user_sys::SwitchtecDevice;
    ///
    /// let device = SwitchtecDevice::new("/dev/pciswitch0");
    /// // OR
    /// let mut path = std::path::PathBuf::from("/dev");
    /// path.push("pciswitch1");
    /// let device = SwitchtecDevice::new(&path);
    /// ```
    pub fn new(path: T) -> Self {
        Self { path_ref: path }
    }

    /// Open the Switchtec PCIe Switch character device, returning
    /// a `SwitchtecDeviceGuard` that can be used to pass into
    /// `switchtec-user` C library functions
    ///
    /// ```no_run
    /// use switchtec_user_sys::{switchtec_die_temp, SwitchtecDevice};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let device = SwitchtecDevice::new("/dev/pciswitch0").open()?;
    ///
    /// // unsafe because `switchtec_die_temp` is an extern function
    /// let temperature = unsafe { switchtec_die_temp(*device) };
    /// println!("Temperature: {temperature}");
    /// // Switchtec device is closed with `device` goes out of scope
    /// # Ok(())
    /// }
    /// ```
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

/// Represents an open Switchtec PCI Switch device that can be passed into `switchtec-user` C library functions
///
/// Closes the Switchtec character device when this goes out of scope
#[must_use]
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
        // SAFETY: Checking that the returned `dev` is not null prior to successfully returning
        // a valid `Self` struct
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
        // SAFETY: SwitchtecDeviceGuard is only successfully constructed if the `inner` `switchtec_dev`
        // is not null;
        unsafe {
            switchtec_close(self.inner);
        }
    }
}
