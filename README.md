# os-release

Rust crate that provides a type for parsing the `/etc/os-release` file, or any file with
an `os-release`-like format.

```rust
extern crate os_release;

use os_release::OsRelease;
use std::io;

pub fn main() -> io::Result<()> {
    let release = OsRelease::new()?;
    println!("{:#?}", release);
    Ok(())
}
```