//! This is an example of using the raw, extern FFI functions from the `switchtec-user` library. Note
//! that all device opening/closing is explicit.
//! Inspired by the [`temp.c` example](https://github.com/Microsemi/switchtec-user/blob/master/examples/temp.c)

use std::env;
use std::ffi::{CStr, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use switchtec_user_sys::{
    switchtec_close, switchtec_dev, switchtec_die_temp, switchtec_open, switchtec_strerror,
};

fn get_temperature(dev: *mut switchtec_dev) -> f32 {
    unsafe { switchtec_die_temp(dev) }
}

fn main() -> anyhow::Result<()> {
    let path: PathBuf = env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "/dev/pciswitch0".to_owned())
        .into();
    let cpath = CString::new(path.as_os_str().as_bytes())?;

    let dev = unsafe { switchtec_open(cpath.as_ptr()) };
    // Must check that the device returned is not null. If it is null an error occurred which
    // is exposed in `switchtec_perror()` and `switchtec_strerror()`
    if dev.is_null() {
        let err = unsafe { CStr::from_ptr(switchtec_strerror()).to_owned() };
        anyhow::bail!(
            "Unable to open switchtec device at {}: {}",
            path.display(),
            err.into_string()?
        );
    }

    // Don't bubble up Errors with '?', since we still have to manually close the device
    let temp = get_temperature(dev);
    // Switchtec returns 100ths of a degree C
    println!("Temp: {}C", temp / 100.0);

    unsafe {
        switchtec_close(dev);
    }

    Ok(())
}
