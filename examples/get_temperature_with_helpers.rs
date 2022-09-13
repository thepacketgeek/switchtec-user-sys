//! This is an example of using the `SwitchtecDevice` helper struct provided in [`lib.rs`] for device
//! opening & closing

use std::env;

use switchtec_user_sys::{switchtec_die_temp, SwitchtecDevice};

fn main() -> anyhow::Result<()> {
    let path = env::args()
        .skip(1)
        .next()
        .unwrap_or_else(|| "/dev/pciswitch0".to_owned());
    let dev = SwitchtecDevice::open(path)?;
    unsafe {
        let temp = switchtec_die_temp(*dev);
        println!("Temperature: {}", temp);
    }

    Ok(())
}
