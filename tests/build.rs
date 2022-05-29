// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::path::Path;

use anyhow::Result;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

use rustsourcebundler::Bundler;

const INPUT_DIR: &'static str = "tests/testdata/input";

#[test]
fn build_hello_world() -> Result<()> {
    build("hello-world")
}

#[test]
fn build_basic() -> Result<()> {
    build("basic")
}

fn build(testname: &str) -> Result<()> {
    // Create the build directory, with an src subdir:
    let input_path_str = format!("{}/{}/src/main.rs", INPUT_DIR, testname);
    let input_path = Path::new(&input_path_str);
    let tempdir = TempDir::new()?;
    fs::create_dir_all(tempdir.path().join("src"))?;
    // Bundle the test input there:
    let output_path = tempdir.path().join("src/main.rs");
    let mut bundler = Bundler::new(&input_path, &output_path);
    bundler.crate_name(testname);
    bundler.run();
    // Create Cargo.toml:
    let mut fd = fs::File::create(tempdir.path().join("Cargo.toml"))?;
    write!(
        fd,
        "[package]\nname = \"{}\"\nversion = \"0.0.0\"\n",
        testname
    )?;
    fd.flush()?;
    // cargo check it:
    let result = Command::new("cargo")
        .arg("check")
        .arg("-q")
        .current_dir(tempdir.path())
        .spawn()?
        .wait()?;
    assert!(result.success());
    Ok(())
}
