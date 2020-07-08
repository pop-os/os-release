extern crate os_release;

use os_release::OsRelease;
use std::io;

pub fn main() -> io::Result<()> {
    let release = OsRelease::new_from("examples/pop_cosmic")?;
    println!("{:#?}", release);
    Ok(())
}
