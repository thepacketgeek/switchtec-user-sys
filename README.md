# switchtec-user-sys
A Rust wrapper for the [`switchtec-user`](https://github.com/Microsemi/switchtec-user) library.

Details and usage instructions for the `switchtec-user` library can be found [here](https://microsemi.github.io/switchtec-user/index.html).


# Example Usage
```rust
use std::env;

use switchtec_user_sys::{switchtec_die_temp, SwitchtecDevice};

fn main() -> anyhow::Result<()> {
    let path = env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "/dev/pciswitch0".to_owned());

    let dev = SwitchtecDevice::new(path).open()?;

    unsafe {
        let temp = switchtec_die_temp(*dev);
        println!("Temperature: {}", temp);
    }

    Ok(())
}
```

# License

`switchtec-user-sys` is both MIT and Apache License, Version 2.0 licensed, as found
in the LICENSE-MIT and LICENSE-APACHE files.