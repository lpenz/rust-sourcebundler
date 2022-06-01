// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use cargo_toml;
use lazy_static::lazy_static;
use regex::Regex;

const LIBRS_FILENAME: &str = "src/lib.rs";
lazy_static! {
    static ref COMMENT_RE: Regex = source_line_regex(r" ").unwrap();
    static ref WARN_RE: Regex = source_line_regex(r" #!\[warn\(.*").unwrap();
    static ref USECRATE_RE: Regex = source_line_regex(r" use  crate::(?P<submod>.*);$").unwrap();
    static ref MINIFY_RE: Regex = Regex::new(r"^\s*(?P<contents>.*)\s*$").unwrap();
}

pub struct Bundler<'a> {
    binrs_filename: &'a Path,
    bundle_filename: Option<&'a Path>,
    bundle_file: Box<dyn Write>,
    basedir: PathBuf,
    _crate_name: String,
    skip_use: HashSet<String>,
    minify: bool,
}

/// Defines a regex to match a line of rust source.
/// Uses a shorthand where "  " = "\s+" and " " = "\s*"
fn source_line_regex<S: AsRef<str>>(source_regex: S) -> Result<Regex> {
    Regex::new(
        format!(
            "^{}(?://.*)?$",
            source_regex
                .as_ref()
                .replace("  ", r"\s+")
                .replace(' ', r"\s*")
        )
        .as_str(),
    )
    .map_err(|e| anyhow!(e))
}

impl<'a> Bundler<'a> {
    pub fn new(binrs_filename: &'a Path, bundle_filename: &'a Path) -> Bundler<'a> {
        let mut bundler = Self::new_fd(
            binrs_filename,
            Box::new(BufWriter::new(File::create(&bundle_filename).unwrap())),
        );
        bundler.bundle_filename = Some(bundle_filename);
        bundler
    }

    pub fn new_fd(binrs_filename: &'a Path, bundle_file: Box<dyn Write>) -> Bundler<'a> {
        Bundler {
            binrs_filename,
            bundle_filename: None,
            bundle_file,
            basedir: PathBuf::default(),
            _crate_name: String::from(""),
            skip_use: HashSet::new(),
            minify: false,
        }
    }

    pub fn minify_set(&mut self, enable: bool) {
        self.minify = enable;
    }

    pub fn crate_name(&mut self, name: &'a str) {
        self._crate_name = String::from(name);
    }

    fn do_run(&mut self) -> Result<()> {
        self.basedir = PathBuf::from(
            self.binrs_filename
                .ancestors()
                .find(|a| a.join("Cargo.toml").is_file())
                .ok_or_else(|| {
                    anyhow!(
                        "could not find Cargo.toml in ancestors of {:?}",
                        self.binrs_filename
                    )
                })?,
        );
        let cargo_filename = self.basedir.join("Cargo.toml");
        let cargo = cargo_toml::Manifest::from_path(&cargo_filename)?;
        self._crate_name = cargo
            .package
            .ok_or_else(|| anyhow!("Could not get crate name from {}", cargo_filename.display()))?
            .name;
        self.binrs()?;
        if let Some(bundle_filename) = self.bundle_filename {
            println!("rerun-if-changed={}", bundle_filename.display());
        }
        self.bundle_file.flush()?;
        Ok(())
    }

    pub fn run(mut self) {
        self.do_run().unwrap();
    }

    /// From the file that has the main() function, expand "extern
    /// crate <_crate_name>" into lib.rs contents, and smartly skips
    /// "use <_crate_name>::" lines.
    fn binrs(&mut self) -> Result<()> {
        let bin_fd = File::open(self.binrs_filename)?;
        let mut bin_reader = BufReader::new(&bin_fd);

        let extcrate_re = source_line_regex(format!(r" extern  crate  {} ; ", self._crate_name))?;
        let useselfcrate_re =
            source_line_regex(format!(r" use  (?P<submod>{}::.*) ; ", self._crate_name))?;

        let mut line = String::new();
        while bin_reader.read_line(&mut line)? > 0 {
            line.truncate(line.trim_end().len());
            if COMMENT_RE.is_match(&line) || WARN_RE.is_match(&line) {
            } else if extcrate_re.is_match(&line) {
                writeln!(self.bundle_file, "pub mod {} {{", self._crate_name)?;
                self.librs()?;
                writeln!(self.bundle_file, "}}")?;
            } else if let Some(cap) = useselfcrate_re.captures(&line) {
                let submod = cap
                    .name("submod")
                    .ok_or_else(|| anyhow!("capture not found"))?
                    .as_str();
                writeln!(self.bundle_file, "use ::{};", submod)?;
            } else {
                self.write_line(&line)?;
            }
            line.clear();
        }
        Ok(())
    }

    /// Expand lib.rs contents and "pub mod <>;" lines.
    fn librs(&mut self) -> Result<()> {
        let lib_fd = File::open(self.basedir.join(LIBRS_FILENAME))?;
        let mut lib_reader = BufReader::new(&lib_fd);

        let mod_re = source_line_regex(r" (pub  )?mod  (?P<m>.+) ; ")?;

        let mut line = String::new();
        while lib_reader.read_line(&mut line)? > 0 {
            line.pop();
            if COMMENT_RE.is_match(&line) || WARN_RE.is_match(&line) {
            } else if let Some(cap) = mod_re.captures(&line) {
                let modname = cap
                    .name("m")
                    .ok_or_else(|| anyhow!("capture not found"))?
                    .as_str();
                if modname != "tests" {
                    self.usemod(modname, modname, modname, 1)?;
                }
            } else {
                self.write_line(&line)?;
            }
            line.clear(); // clear to reuse the buffer
        }
        Ok(())
    }

    /// Called to expand random .rs files from lib.rs. It recursivelly
    /// expands further "pub mod <>;" lines and updates the list of
    /// "use <>;" lines that have to be skipped.
    fn usemod(
        &mut self,
        mod_name: &str,
        mod_path: &str,
        mod_import: &str,
        lvl: usize,
    ) -> Result<()> {
        let mod_filenames0 = vec![
            format!("src/{}.rs", mod_path),
            format!("src/{}/mod.rs", mod_path),
        ];
        let mod_fd = mod_filenames0
            .iter()
            .map(|fn0| {
                let mod_filename = self.basedir.join(&fn0);
                File::open(mod_filename)
            })
            .find(|fd| fd.is_ok())
            .ok_or_else(|| anyhow!("no mod file found"))??;
        let mut mod_reader = BufReader::new(mod_fd);

        let mod_re = source_line_regex(r" (pub  )?mod  (?P<m>.+) ; ")?;

        let mut line = String::new();

        writeln!(self.bundle_file, "pub mod {} {{", mod_name)?;
        self.skip_use.insert(String::from(mod_import));

        while mod_reader.read_line(&mut line)? > 0 {
            line.truncate(line.trim_end().len());
            if COMMENT_RE.is_match(&line) || WARN_RE.is_match(&line) {
            } else if let Some(cap) = USECRATE_RE.captures(&line) {
                let submodname = cap
                    .name("submod")
                    .ok_or_else(|| anyhow!("capture not found"))?
                    .as_str();
                write!(self.bundle_file, "use ")?;
                for _ in 0..lvl {
                    write!(self.bundle_file, "super::")?;
                }
                writeln!(self.bundle_file, "{};", submodname)?;
            } else if let Some(cap) = mod_re.captures(&line) {
                let submodname = cap
                    .name("m")
                    .ok_or_else(|| anyhow!("capture not found"))?
                    .as_str();
                if submodname != "tests" {
                    let submodfile = format!("{}/{}", mod_path, submodname);
                    let submodimport = format!("{}::{}", mod_import, submodname);
                    self.usemod(
                        submodname,
                        submodfile.as_str(),
                        submodimport.as_str(),
                        lvl + 1,
                    )?;
                }
            } else {
                self.write_line(&line)?;
            }
            line.clear(); // clear to reuse the buffer
        }

        writeln!(self.bundle_file, "}}")?;

        Ok(())
    }

    fn write_line(&mut self, line: &str) -> Result<()> {
        if self.minify {
            writeln!(
                self.bundle_file,
                "{}",
                MINIFY_RE.replace_all(line, "$contents")
            )
        } else {
            writeln!(self.bundle_file, "{}", line)
        }
        .map_err(|e| anyhow!(e))
    }
}
