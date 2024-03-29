// Based on the tests of https://github.com/Endle/rust-bundler-cp

use std::path::Path;

use anyhow::Result;
use goldenfile::Mint;

use rustsourcebundler::Bundler;

const INPUT_DIR: &'static str = "tests/testdata/input";
const OUTPUT_DIR: &'static str = "tests/testdata/output";

#[test]
fn golden_hello_world() -> Result<()> {
    golden("hello-world")
}

#[test]
fn golden_basic() -> Result<()> {
    golden("basic")
}

#[test]
fn golden_usecrate() -> Result<()> {
    golden("usecrate")
}

#[test]
fn golden_complicated() -> Result<()> {
    golden("complicated")
}

fn golden(testname: &str) -> Result<()> {
    let input_path_str = format!("{}/{}/src/main.rs", INPUT_DIR, testname);
    let input_path = Path::new(&input_path_str);
    let output_name = Path::new(testname).with_extension("rs");
    let mut mint = Mint::new(OUTPUT_DIR);
    let golden = mint.new_goldenfile(&output_name)?;
    let mut bundler = Bundler::new_fd(&input_path, Box::new(golden));
    bundler.crate_name(testname);
    bundler.run();
    Ok(())
}
