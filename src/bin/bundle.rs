// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

use rustsourcebundler::Bundler;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Cli {
    /// The top src/main.rs or src/bin/*.rs file to bundle.
    pub main_rs: PathBuf,
    /// The output file, stdout by default.
    pub output: Option<PathBuf>,
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let bundler = if let Some(ref output) = cli.output {
        Bundler::new(&cli.main_rs, output)
    } else {
        Bundler::new_fd(&cli.main_rs, Box::new(std::io::stdout()))
    };
    bundler.run();
    Ok(())
}
