/*!
Use this library in your build.rs to create a single file with all the crate's source code.

That's useful for programming exercise sites that take a single source file.
*/

extern crate cargo_metadata;
extern crate quote;
extern crate rustfmt;
extern crate syn;

use quote::ToTokens;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use syn::visit_mut::VisitMut;

pub mod rust_bundler;

const LIBRS_FILENAME: &str = "src/lib.rs";

#[derive(Debug, Clone)]
pub struct Bundler<'a> {
    binrs_filename: &'a Path,
    bundle_filename: &'a Path,
    librs_filename: &'a Path,
    _crate_name: &'a str,
}

/// Defines a regex to match a line of rust source.
/// Uses a shorthand where "  " = "\s+" and " " = "\s*"
fn source_line_regex<S: AsRef<str>>(source_regex: S) -> Regex {
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
    .unwrap()
}

impl<'a> Bundler<'a> {
    pub fn new(binrs_filename: &'a Path, bundle_filename: &'a Path) -> Bundler<'a> {
        Bundler {
            binrs_filename,
            bundle_filename,
            librs_filename: Path::new(LIBRS_FILENAME),
            _crate_name: "",
        }
    }

    pub fn crate_name(&mut self, name: &'a str) {
        self._crate_name = name;
    }

    pub fn run(&mut self) {
        let base_path = self
            .librs_filename
            .parent()
            .expect("lib.src_path has no parent");
        let code = rust_bundler::read_file(self.binrs_filename)
            .expect("failed to read binary target source");
        let mut file = syn::parse_file(&code).expect("failed to parse binary target source");
        rust_bundler::Expander {
            base_path,
            crate_name: self._crate_name,
        }
        .visit_file_mut(&mut file);
        let code = file.into_tokens().to_string();
        let codepretty = rust_bundler::prettify(code);
        let mut o = File::create(&self.bundle_filename)
            .unwrap_or_else(|_| panic!("error creating {}", &self.bundle_filename.display()));
        writeln!(&mut o, "{}", codepretty).expect("error writing file");
        println!("rerun-if-changed={}", self.bundle_filename.display());
    }
}
