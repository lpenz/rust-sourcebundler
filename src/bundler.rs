// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use regex::Regex;

const LIBRS_FILENAME: &str = "src/lib.rs";
lazy_static! {
    static ref COMMENT_RE: Regex = source_line_regex(r" ").unwrap();
    static ref WARN_RE: Regex = source_line_regex(r" #!\[warn\(.*").unwrap();
    static ref MINIFY_RE: Regex = Regex::new(r"^\s*(?P<contents>.*)\s*$").unwrap();
}

#[derive(Debug)]
pub struct Bundler<'a> {
    binrs_filename: &'a Path,
    bundle_filename: &'a Path,
    bundle_file: Option<File>,
    librs_filename: &'a Path,
    basedir: PathBuf,
    _crate_name: &'a str,
    skip_use: HashSet<String>,
    minify: bool,
}

impl<'a> Default for Bundler<'a> {
    fn default() -> Self {
        Bundler {
            binrs_filename: Path::new(""),
            bundle_filename: Path::new(""),
            bundle_file: None,
            librs_filename: Path::new(LIBRS_FILENAME),
            basedir: PathBuf::default(),
            _crate_name: "",
            skip_use: HashSet::new(),
            minify: false,
        }
    }
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
        Bundler {
            binrs_filename,
            bundle_filename,
            ..Default::default()
        }
    }

    pub fn new_fd(binrs_filename: &'a Path, bundle_file: File) -> Bundler<'a> {
        Bundler {
            binrs_filename,
            bundle_file: Some(bundle_file),
            ..Default::default()
        }
    }

    pub fn minify_set(&mut self, enable: bool) {
        self.minify = enable;
    }

    pub fn crate_name(&mut self, name: &'a str) {
        self._crate_name = name;
    }

    fn do_run(&mut self) -> Result<()> {
        let mut o = {
            if let Some(o) = self.bundle_file.take() {
                o
            } else {
                File::create(&self.bundle_filename)?
            }
        };
        self.basedir = PathBuf::from(
            self.binrs_filename
                .ancestors()
                .find(|a| a.join("Cargo.toml").is_file())
                .ok_or(anyhow!(
                    "could not find Cargo.toml in ancestors of {:?}",
                    self.binrs_filename
                ))?,
        );
        self.binrs(&mut o)?;
        println!("rerun-if-changed={}", self.bundle_filename.display());
        Ok(())
    }

    pub fn run(&mut self) {
        self.do_run().unwrap();
    }

    /// From the file that has the main() function, expand "extern
    /// crate <_crate_name>" into lib.rs contents, and smartly skips
    /// "use <_crate_name>::" lines.
    fn binrs(&mut self, mut o: &mut File) -> Result<()> {
        let bin_fd = File::open(self.binrs_filename)?;
        let mut bin_reader = BufReader::new(&bin_fd);

        let extcrate_re = source_line_regex(format!(
            r" extern  crate  {} ; ",
            String::from(self._crate_name)
        ))?;
        let usecrate_re = source_line_regex(
            format!(r" use  {} :: (.*) ; ", String::from(self._crate_name)).as_str(),
        )?;

        let mut line = String::new();
        while bin_reader.read_line(&mut line)? > 0 {
            line.truncate(line.trim_end().len());
            if COMMENT_RE.is_match(&line) || WARN_RE.is_match(&line) {
            } else if extcrate_re.is_match(&line) {
                self.librs(o)?;
            } else if let Some(cap) = usecrate_re.captures(&line) {
                let moduse = cap
                    .get(1)
                    .ok_or_else(|| anyhow!("capture not found"))?
                    .as_str();
                if !self.skip_use.contains(moduse) {
                    writeln!(&mut o, "use {};", moduse)?;
                }
            } else {
                self.write_line(o, &line)?;
            }
            line.clear();
        }
        Ok(())
    }

    /// Expand lib.rs contents and "pub mod <>;" lines.
    fn librs(&mut self, o: &mut File) -> Result<()> {
        let lib_fd = File::open(self.basedir.join(self.librs_filename))?;
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
                    self.usemod(o, modname, modname, modname)?;
                }
            } else {
                self.write_line(o, &line)?;
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
        mut o: &mut File,
        mod_name: &str,
        mod_path: &str,
        mod_import: &str,
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

        writeln!(&mut o, "pub mod {} {{", mod_name)?;
        self.skip_use.insert(String::from(mod_import));

        while mod_reader.read_line(&mut line)? > 0 {
            line.truncate(line.trim_end().len());
            if COMMENT_RE.is_match(&line) || WARN_RE.is_match(&line) {
            } else if let Some(cap) = mod_re.captures(&line) {
                let submodname = cap
                    .name("m")
                    .ok_or_else(|| anyhow!("capture not found"))?
                    .as_str();
                if submodname != "tests" {
                    let submodfile = format!("{}/{}", mod_path, submodname);
                    let submodimport = format!("{}::{}", mod_import, submodname);
                    self.usemod(o, submodname, submodfile.as_str(), submodimport.as_str())?;
                }
            } else {
                self.write_line(o, &line)?;
            }
            line.clear(); // clear to reuse the buffer
        }

        writeln!(&mut o, "}}")?;

        Ok(())
    }

    fn write_line(&self, mut o: &mut File, line: &str) -> Result<()> {
        if self.minify {
            writeln!(&mut o, "{}", MINIFY_RE.replace_all(line, "$contents"))
        } else {
            writeln!(&mut o, "{}", line)
        }
        .map_err(|e| anyhow!(e))
    }
}
