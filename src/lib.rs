#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(clippy::missing_safety_doc)]
#![doc = include_str!("../README.md")]

use std::ffi::{CStr, CString};
use std::fmt;
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// `SwitchtecDevice` offers an safer way to work with the underlying [`switchtec_dev`] and
/// represents an open Switchtec PCI Switch device that can be passed into `switchtec-user` C library functions
///
/// - [`SwitchtecDevice`] closes the Switchtec character device when it goes out of scope
pub struct SwitchtecDevice {
    inner: *mut switchtec_dev,
}

impl SwitchtecDevice {
    /// Open the Switchtec PCIe Switch character device at the given `path`,
    /// returning a `SwitchtecDevice` that can be used to pass into
    /// `switchtec-user` C library functions
    ///
    /// ```no_run
    /// use switchtec_user_sys::{switchtec_die_temp, SwitchtecDevice};
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let device = SwitchtecDevice::open("/dev/pciswitch0")?;
    ///
    /// // SAFETY: We know that device holds a valid/open switchtec device
    /// let temperature = unsafe { switchtec_die_temp(*device) };
    /// println!("Temperature: {temperature}");
    /// // Switchtec device is closed with `device` goes out of scope
    /// # Ok(())
    /// }
    /// ```
    pub fn open<T: AsRef<Path>>(path: T) -> io::Result<Self> {
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
                Err(get_switchtec_error())
            } else {
                Ok(Self { inner: dev })
            }
        }
    }

    /// Get the device name (E.g. "pciswitch0" in "/dev/pciswitch0")
    ///
    /// This can fail if the device name is not valid UTF-8
    ///
    /// <https://microsemi.github.io/switchtec-user/group__Device.html#ga8d416a587f5e37e818ee937bd0c0dab1>
    pub fn name(&self) -> io::Result<String> {
        // SAFETY: We know that device holds a valid/open switchtec device
        let device_name = unsafe { switchtec_name(self.inner) };
        if device_name.is_null() {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "no device name returned",
            ))
        } else {
            device_name.as_string()
        }
    }

    /// Get the PCIe generation of the device
    ///
    /// <https://microsemi.github.io/switchtec-user/group__Device.html#ga9eab19beb39d2104b5defd28787177ae>
    pub fn boot_phase(&self) -> switchtec_boot_phase {
        // SAFETY: We know that device holds a valid/open switchtec device
        unsafe { switchtec_boot_phase(self.inner) }
    }

    /// Get the firmware version as a user readable string
    ///
    /// This can fail if the firmware version is not valid UTF-8
    ///
    /// <https://microsemi.github.io/switchtec-user/group__Device.html#gad16f110712bd23170ad69450c361122e>
    pub fn firmware_version(&self) -> io::Result<String> {
        const buf_size: usize = 64;
        let mut buf = MaybeUninit::<[u8; buf_size]>::uninit();
        // SAFETY: We know that device holds a valid/open switchtec device
        unsafe {
            let len =
                switchtec_get_fw_version(self.inner, buf.as_mut_ptr() as *mut _, buf_size as u64);
            if len.is_negative() {
                Err(get_switchtec_error())
            } else {
                buf_to_string(&buf.assume_init())
            }
        }
    }

    /// Get the PCIe generation of the device
    ///
    /// <https://microsemi.github.io/switchtec-user/group__Device.html#gab9f59d48c410e8dde13acdc519943a26>
    pub fn generation(&self) -> switchtec_gen {
        // SAFETY: We know that device holds a valid/open switchtec device
        unsafe { switchtec_gen(self.inner) }
    }

    /// Get the partition of the device
    ///
    /// <https://microsemi.github.io/switchtec-user/group__Device.html#gac70f47bb86ac6ba1666446f27673cdcf>
    pub fn partition(&self) -> i32 {
        // SAFETY: We know that device holds a valid/open switchtec device
        unsafe { switchtec_partition(self.inner) }
    }
}

impl fmt::Debug for SwitchtecDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SwitchtecDevice")
            .field("name", &self.name().as_deref().unwrap_or("unknown"))
            .finish()
    }
}

impl std::ops::Deref for SwitchtecDevice {
    type Target = *mut switchtec_dev;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::Drop for SwitchtecDevice {
    fn drop(&mut self) {
        // SAFETY: SwitchtecDevice is only successfully constructed if the `inner` `switchtec_dev`
        // is not null;
        unsafe {
            switchtec_close(self.inner);
        }
    }
}

pub trait CStrExt {
    /// Convert a C-style string (E.g. `char*`) to a Rust [`String`]
    ///
    /// Returns an [`io::Error`] if the string pointer is null or cannot be
    fn as_string(&self) -> io::Result<String>;
}

impl CStrExt for *const i8 {
    /// Copy a C-style `*const i8` string to a [`String`]
    ///
    /// ```
    /// use switchtec_user_sys::CStrExt;
    /// # use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cstr = CString::new(*b"hello")?;
    /// // This is a type you might receive from an extern "C" function:
    /// let str_value: *const i8 = cstr.as_ptr() as *const i8;
    ///
    /// let rust_string: String = str_value.as_string()?;
    /// assert_eq!(&rust_string, "hello");
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn as_string(&self) -> io::Result<String> {
        cstr_to_string(*self)
    }
}

impl CStrExt for *mut i8 {
    /// Copy a C-style `*mut i8` string to a [`String`]
    ///
    /// ```
    /// use switchtec_user_sys::CStrExt;
    /// # use std::ffi::CString;
    ///
    /// # fn main() -> anyhow::Result<()> {
    /// let cstr = CString::new(*b"hello")?;
    /// // This is a type you might receive from an extern "C" function:
    /// let str_value: *mut i8 = cstr.as_ptr() as *mut i8;
    ///
    /// let rust_string: String = str_value.as_string()?;
    /// assert_eq!(&rust_string, "hello");
    ///
    /// # Ok(())
    /// # }
    /// ```
    fn as_string(&self) -> io::Result<String> {
        cstr_to_string(*self)
    }
}

fn cstr_to_string(cstr: *const i8) -> io::Result<String> {
    if cstr.is_null() {
        Ok("".to_owned())
    } else {
        // SAFETY: cstr has been checked for null, we can safely dereference
        unsafe {
            let s = CStr::from_ptr(cstr).to_owned();
            s.into_string().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("error decoding String from {cstr:?}: {e}"),
                )
            })
        }
    }
}

/// Parse a String from a buffer that may have tail-padding
fn buf_to_string(buf: &[u8]) -> io::Result<String> {
    let valid_bytes: Vec<u8> = buf
        .iter()
        // Filter out null bytes
        .take_while(|b| b != &&0)
        .copied()
        .collect();
    let cstring = CString::new(valid_bytes)?;
    cstring.into_raw().as_string()
}

fn get_switchtec_error() -> io::Error {
    // SAFETY: We're checking that the returned char* is not null
    let err_message = unsafe {
        // https://microsemi.github.io/switchtec-user/group__Device.html#ga595e1d62336ba76c59344352c334fa18
        let err_str = switchtec_strerror();
        if err_str.is_null() {
            return io::Error::new(io::ErrorKind::Other, "Unknown error".to_owned());
        }
        err_str
            .as_string()
            .unwrap_or_else(|_| "Unknown error".to_owned())
    };
    io::Error::new(io::ErrorKind::Other, err_message)
}

#[test]
fn test_buf_to_string() {
    let buf = [51, 46, 55, 48, 32, 66, 48, 52, 70, 0, 0, 0, 0, 0, 0, 0];
    assert_eq!(&buf_to_string(&buf).unwrap(), "3.70 B04F");
}
