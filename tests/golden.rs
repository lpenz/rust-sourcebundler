// Based on the tests of https://github.com/Endle/rust-bundler-cp

use std::path::{Path, PathBuf};

use anyhow::Result;
use goldenfile::Mint;

use rustsourcebundler::Bundler;

const INPUT_DIR: &'static str = "tests/testdata/input";
const OUTPUT_DIR: &'static str = "tests/testdata/output";

#[test]
fn hello_world() -> Result<()> {
    let mut mint = Mint::new(OUTPUT_DIR);
    golden(&mut mint, "hello-world")
}

fn golden(mint: &mut Mint, testname: &str) -> Result<()> {
    let output_name = Path::new(testname).with_extension("rs");
    let input_path = {
        let mut p = PathBuf::from(INPUT_DIR);
        p.push(testname);
        p.push("src");
        p.push("main.rs");
        p
    };
    let golden = mint.new_goldenfile(&output_name)?;
    let mut bundler = Bundler::new_fd(&input_path, golden);
    bundler.crate_name(testname);
    bundler.run();
    Ok(())
}
