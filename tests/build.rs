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
fn build_original_hello_world() -> Result<()> {
    build_original("hello-world")
}

#[test]
fn build_bundle_hello_world() -> Result<()> {
    build_bundle("hello-world")
}

#[test]
fn build_original_basic() -> Result<()> {
    build_original("basic")
}

#[test]
fn build_bundle_basic() -> Result<()> {
    build_bundle("basic")
}

#[test]
fn build_original_usecrate() -> Result<()> {
    build_original("usecrate")
}

#[test]
fn build_bundle_usecrate() -> Result<()> {
    build_bundle("usecrate")
}

#[test]
fn build_original_complicated() -> Result<()> {
    build_original("complicated")
}

#[test]
fn build_bundle_complicated() -> Result<()> {
    build_bundle("complicated")
}

fn build_original(testname: &str) -> Result<()> {
    let input_path = Path::new(INPUT_DIR).join(testname);
    let targetdir = TempDir::new()?;
    let result = Command::new("cargo")
        .arg("check")
        .arg("-q")
        .current_dir(input_path)
        .env("CARGO_TARGET_DIR", targetdir.path())
        .spawn()?
        .wait()?;
    assert!(result.success());
    Ok(())
}

fn build_bundle(testname: &str) -> Result<()> {
    // Create the build directory, with an src subdir:
    let input_path_str = format!("{}/{}/src/main.rs", INPUT_DIR, testname);
    let input_path = Path::new(&input_path_str);
    let tempdir = TempDir::new()?;
    fs::create_dir_all(tempdir.path().join("src"))?;
    // Bundle the test input there:
    let output_path = tempdir.path().join("src/main.rs");
    let bundler = Bundler::new(&input_path, &output_path);
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
