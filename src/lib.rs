/*!
Use this library in your build.rs to create a single file with all the crate's source code.

That's useful for programming exercise sites that take a single source file.
*/

use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

extern crate regex;
use regex::Regex;

const LIBRS_FILENAME: &'static str = "src/lib.rs";

#[derive(Debug, Clone)]
pub struct Bundler<'a> {
    binrs_filename: &'a Path,
    bundle_filename: &'a Path,
    librs_filename: &'a Path,
    comment_re: Regex,
    _crate_name: &'a str,
    skip_use: HashSet<String>,
    minify_re: Option<Regex>,
}

impl<'a> Bundler<'a> {
    pub fn new(binrs_filename: &'a Path, bundle_filename: &'a Path) -> Bundler<'a> {
        Bundler {
            binrs_filename: binrs_filename,
            bundle_filename: bundle_filename,
            librs_filename: Path::new(LIBRS_FILENAME),
            comment_re: Regex::new(r"^\s*//").unwrap(),
            _crate_name: "",
            skip_use: HashSet::new(),
            minify_re: None,
        }
    }

    pub fn minify_set(&mut self, enable: bool) {
        self.minify_re = if enable {
            Some(Regex::new(r"^\s*(?P<contents>.*)\s*$").unwrap())
        } else {
            None
        };
    }

    pub fn crate_name(&mut self, name: &'a str) {
        self._crate_name = name;
    }

    pub fn run(&mut self) {
        let mut o = File::create(&self.bundle_filename).expect(&format!(
            "error creating {}",
            &self.bundle_filename.display()
        ));
        self.binrs(&mut o).expect(&format!(
            "error creating bundle {} for {}",
            self.bundle_filename.display(),
            self.binrs_filename.display()
        ));
        println!("rerun-if-changed={}", self.bundle_filename.display());
    }

    /// From the file that has the main() function, expand "extern
    /// crate <_crate_name>" into lib.rs contents, and smartly skips
    /// "use <_crate_name>::" lines.
    fn binrs(&mut self, mut o: &mut File) -> Result<(), io::Error> {
        let bin_fd = File::open(self.binrs_filename)?;
        let mut bin_reader = BufReader::new(&bin_fd);

        let extcrate_re =
            Regex::new(format!(r"^extern crate {};$", String::from(self._crate_name)).as_str())
                .unwrap();
        let usecrate_re =
            Regex::new(format!(r"^use {}::(.*);$", String::from(self._crate_name)).as_str())
                .unwrap();

        let mut line = String::new();
        while bin_reader.read_line(&mut line).unwrap() > 0 {
            line.pop();
            if self.comment_re.is_match(&line) {
            } else if extcrate_re.is_match(&line) {
                self.librs(o)?;
            } else if let Some(cap) = usecrate_re.captures(&line) {
                let moduse = cap.get(1).unwrap().as_str();
                if !self.skip_use.contains(moduse) {
                    writeln!(&mut o, "use {};", moduse)?;
                }
            } else {
                self.write_line(&mut o, &line)?;
            }
            line.clear();
        }
        Ok(())
    }

    /// Expand lib.rs contents and "pub mod <>;" lines.
    fn librs(&mut self, mut o: &mut File) -> Result<(), io::Error> {
        let lib_fd = File::open(self.librs_filename).expect("could not open lib.rs");
        let mut lib_reader = BufReader::new(&lib_fd);

        let mod_re = Regex::new(r"^\s*pub mod (.+);$").unwrap();

        let mut line = String::new();
        while lib_reader.read_line(&mut line).unwrap() > 0 {
            line.pop();
            if self.comment_re.is_match(&line) {
            } else if let Some(cap) = mod_re.captures(&line) {
                let modname = cap.get(1).unwrap().as_str();
                if modname != "tests" {
                    self.usemod(o, modname, modname)?;
                }
            } else {
                self.write_line(&mut o, &line)?;
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
    ) -> Result<(), io::Error> {
        let mod_filenames0 = vec![
            format!("src/{}.rs", mod_name),
            format!("src/{}/mod.rs", mod_name),
        ];
        let mod_fd = mod_filenames0
            .iter()
            .map(|fn0| {
                let mod_filename = Path::new(&fn0);
                File::open(mod_filename)
            })
            .filter(|fd| fd.is_ok())
            .next();
        assert!(
            mod_fd.is_some(),
            format!("could not find file for module {}", mod_name)
        );
        let mut mod_reader = BufReader::new(mod_fd.unwrap().unwrap());

        let mod_re = Regex::new(r"^\s*pub mod (.+);$").unwrap();

        let mut line = String::new();

        writeln!(&mut o, "pub mod {} {{", mod_name)?;
        self.skip_use.insert(String::from(mod_path));

        while mod_reader.read_line(&mut line).unwrap() > 0 {
            line.pop();
            if self.comment_re.is_match(&line) {
            } else if let Some(cap) = mod_re.captures(&line) {
                let submodname = cap.get(1).unwrap().as_str();
                if submodname != "tests" {
                    let submodpath = format!("{}::{}", mod_path, submodname);
                    self.usemod(o, submodname, submodpath.as_str())?;
                }
            } else {
                self.write_line(&mut o, &line)?;
            }
            line.clear(); // clear to reuse the buffer
        }

        writeln!(&mut o, "}}")?;

        Ok(())
    }

    fn write_line(&self, mut o: &mut File, line: &str) -> Result<(), io::Error> {
        if let Some(ref minify_re) = self.minify_re {
            writeln!(
                &mut o,
                "{}",
                minify_re.replace_all(line, "$contents")
            )
        } else {
            writeln!(&mut o, "{}", line)
        }
    }
}
