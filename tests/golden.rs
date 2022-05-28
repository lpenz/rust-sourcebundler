// Based on the tests of https://github.com/Endle/rust-bundler-cp

use std::path::Path;

use anyhow::Result;
use goldenfile::Mint;

use rustsourcebundler::Bundler;

const INPUT_DIR: &'static str = "tests/testdata/input";
const OUTPUT_DIR: &'static str = "tests/testdata/output";

#[test]
fn hello_world() -> Result<()> {
    golden("hello-world")
}

#[test]
fn basic() -> Result<()> {
    golden("basic")
}

fn golden(testname: &str) -> Result<()> {
    let input_path_str = format!("{}/{}/src/main.rs", INPUT_DIR, testname);
    let input_path = Path::new(&input_path_str);
    let output_name = Path::new(testname).with_extension("rs");
    let mut mint = Mint::new(OUTPUT_DIR);
    let golden = mint.new_goldenfile(&output_name)?;
    let mut bundler = Bundler::new_fd(&input_path, golden);
    bundler.crate_name(testname);
    bundler.run();
    Ok(())
}
