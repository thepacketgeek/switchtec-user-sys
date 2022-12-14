# switchtec-user-sys [![Build](https://img.shields.io/github/workflow/status/thepacketgeek/switchtec-user-sys/ci-build?logo=github)](https://github.com/thepacketgeek/switchtec-user-sys/actions/workflows/ci-build.yml) [![Docs](https://img.shields.io/docsrs/switchtec-user-sys?logo=rust)](https://docs.rs/crate/switchtec-user-sys/latest)

A Rust wrapper for the [`switchtec-user`](https://github.com/Microsemi/switchtec-user) library.

Details and usage instructions for the `switchtec-user` library can be found [here](https://microsemi.github.io/switchtec-user/index.html).


# Example Usage
The examples below use the [`SwitchtecDevice`] and [`CStrExt`] types introduced by this library, including functions from the [`switchtec-user`](https://github.com/Microsemi/switchtec-user) C library that this crate wraps.

## Get pciswitch device name and temperature
Example using the [`switchtec_name`](https://microsemi.github.io/switchtec-user/group__Device.html#ga8d416a587f5e37e818ee937bd0c0dab1) and [`switchtec_die_temp`](https://microsemi.github.io/switchtec-user/group__Misc.html#ga56317f0a31a83eb896e4a987dbd645df) functions provided by the `switchtec-user` library

```rust,no_run
use std::env;

use switchtec_user_sys::{switchtec_die_temp, switchtec_name, SwitchtecDevice, CStrExt};

fn main() -> anyhow::Result<()> {
    let path = env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "/dev/pciswitch0".to_owned());

    let device = SwitchtecDevice::open(path)?;

    // SAFETY: We know that device holds a valid/open switchtec device
    let (device_name, temperature) = unsafe {
        let temp = switchtec_die_temp(*device);
        // `CStrExt` is providing `as_string()` here for the returned C-style char* string
        let name = switchtec_name(*device).as_string()?;
        (name, temp)
    };
    println!("Temperature for {device_name}: {temperature}");

    Ok(())
}
```

## Get status for each port for a pciswitch device
A more complex example using an out-value struct with [`switchtec_status`](https://microsemi.github.io/switchtec-user/group__Device.html#ga780a757b81a704c19217aca00f42b50e)

```rust,no_run
use std::env;
use std::io;
use std::ptr;

use switchtec_user_sys::{switchtec_status, switchtec_status_free, SwitchtecDevice};

fn main() -> anyhow::Result<()> {
    let path: std::path::PathBuf = "/dev/pciswitch1".into();
    let device = SwitchtecDevice::open(&path)?;

    // Response struct out-value, to be populated by `switchtec_status`
    // The struct is the same name as the function, so we access this by its
    // full path in order to keep from having a name conflict
    let mut status: *mut switchtec_user_sys::switchtec_status = ptr::null_mut();

    // SAFETY: We're checking that the returned status is not null, and the `port_count`
    // resp provides how many `switchtec_status` structs are present in the data
    let per_port_status = unsafe {
        // We pass in a pointer (*mut) to the status pointer (*mut)
        let port_count = switchtec_status(*device, ptr::addr_of_mut!(status));
        let resp = if status.is_null() || port_count.is_negative() {
            // Negative value represents an error
            // https://microsemi.github.io/switchtec-user/group__Device.html#ga780a757b81a704c19217aca00f42b50e

            // Don't return this immediately so this function can call switchtec_status_free first
            // - For getting the actual error, consider using `switchtec_user_sys::switchtec_strerror`
            // https://microsemi.github.io/switchtec-user/group__Device.html#ga595e1d62336ba76c59344352c334fa18
            Err(io::Error::new(io::ErrorKind::Other, format!("Unknown error")))
        } else {
            // If the call was successful, create a slice from the populated status array
            // for only as many structs were returned: `port_count`
            let statuses: Vec<_> = std::slice::from_raw_parts(status, port_count as usize)
                .iter()
                .take(port_count as usize)
                .copied()
                .collect();
            Ok(statuses)
        };

        // Must be called after switchtec_status to free allocated status structs
        // https://microsemi.github.io/switchtec-user/group__Device.html#ga742519774cbc236ba2d80a08a7dc6b5f
        switchtec_status_free(status as *mut _, port_count);

        resp
    }?;

    println!("{per_port_status:#?}");
    Ok(())
}
```

# Dependencies

## Bindgen & Clang
This crate uses [`bindgen`](https://github.com/rust-lang/rust-bindgen) which requires Clang to build: [`bindgen` requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)

## Switchtec submodule
In order to build the `bindgen` bindings and link the library to the `switchtec-user` C library, the `switchtec-user` submodule needs to be present. The [`build.rs`](build.rs) script will automatically init the submodule, but you can also do this like:

```ignore
git submodule update --init
```

# License

`switchtec-user-sys` is both MIT and Apache License, Version 2.0 licensed, as found
in the LICENSE-MIT and LICENSE-APACHE files.