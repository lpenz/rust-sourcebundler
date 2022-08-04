[![CI](https://github.com/lpenz/rust-sourcebundler/actions/workflows/ci.yml/badge.svg)](https://github.com/lpenz/rust-sourcebundler/actions/workflows/ci.yml)
[![coveralls](https://coveralls.io/repos/github/lpenz/rust-sourcebundler/badge.svg?branch=main)](https://coveralls.io/github/lpenz/rust-sourcebundler?branch=main)
[![crates.io](https://img.shields.io/crates/v/rustsourcebundler.svg)](https://crates.io/crates/rustsourcebundler)

# rust-sourcebundler

Bundle the source code of a rust cargo crate in a single source file.

Very useful for sending the source code to a competitive programming site that
accept only a single file ([codingame](https://codingame.com), I'm looking at
you) and still keeping the cargo structure locally.


## Usage

Add the following snippet to your *Cargo.toml*:

```toml
[package]
(...)
build = "build.rs"

[build-dependencies]
rustsourcebundler = { git = "https://github.com/lpenz/rust-sourcebundler" }
```

And create the file *build.rs* with the following:

```rust
//! Bundle mybin.rs and the crate libraries into singlefile.rs

use std::path::Path;
extern crate rustsourcebundler;
use rustsourcebundler::Bundler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bundler: Bundler = Bundler::new(Path::new("src/bin/mybin.rs"),
                                            Path::new("src/bin/singlefile.rs"));
    bundler.crate_name("<crate name>");
    bundler.run()?;
    Ok(())
}
```

You can use the code inside the *example* directory of this repository
as a starting point.


## Similar Projects

* [slava-sh/rust-bundler](https://github.com/slava-sh/rust-bundler)
* [Endle/rust-bundler-cp](https://github.com/Endle/rust-bundler-cp)
* [MarcosCosmos/cg-rust-bundler](https://github.com/MarcosCosmos/cg-rust-bundler)
  written in python

