/*!
Use this library in your build.rs to create a single file with all the crate's source code.

That's useful for programming exercise sites that take a single source file.
*/

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::io::BufReader;
use std::io::BufRead;
use std::io;

extern crate regex;
use regex::Regex;


const LIBRS_FILENAME: &'static str = "src/lib.rs";


#[derive(Debug, Clone)]
pub struct Bundler<'a> {
    binrs_filename: &'a Path,
    bundle_filename: &'a Path,
    librs_filename: &'a Path,
}


impl<'a> Bundler<'a> {
    pub fn new(binrs_filename: &'a Path, bundle_filename: &'a Path) -> Bundler<'a> {
        Bundler {
            binrs_filename: binrs_filename,
            bundle_filename: bundle_filename,
            librs_filename: Path::new(LIBRS_FILENAME),
        }
    }


    pub fn run(&mut self) {
        let mut o = File::create(&self.bundle_filename)
            .expect(&format!("error creating {}", &self.bundle_filename.display()));
        self.binrs(&mut o)
            .expect(&format!("error creating bundle {} for {}",
                            self.bundle_filename.display(),
                            self.binrs_filename.display()));
        println!("rerun-if-changed={}", self.bundle_filename.display());
    }


    fn binrs(&mut self, mut o: &mut File) -> Result<(), io::Error> {
        let bin_fd = try!(File::open(self.binrs_filename));
        let mut bin_reader = BufReader::new(&bin_fd);

        let extcrate_re = Regex::new(r"^extern crate codersstrikeback;$").unwrap();
        let usecrate_re = Regex::new(r"^use codersstrikeback::(.*);$").unwrap();

        let mut line = String::new();
        while bin_reader.read_line(&mut line).unwrap() > 0 {
            line.pop();
            if extcrate_re.is_match(&line) {
                try!(self.librs(o));
            } else if let Some(cap) = usecrate_re.captures(&line) {
                try!(writeln!(&mut o, "use {};", cap.get(1).unwrap().as_str()));
            } else {
                try!(writeln!(&mut o, "{}", line.chars().collect::<String>()));
            }
            line.clear();
        }
        Ok(())
    }


    fn librs(&mut self, mut o: &mut File) -> Result<(), io::Error> {
        let lib_fd = File::open(self.librs_filename).expect("could not open lib.rs");
        let mut lib_reader = BufReader::new(&lib_fd);

        let mod_re = Regex::new(r"^\s*pub mod (.+);$").unwrap();

        let mut line = String::new();
        while lib_reader.read_line(&mut line).unwrap() > 0 {
            line.pop();
            if let Some(cap) = mod_re.captures(&line) {
                try!(self.usemod(o, cap.get(1).unwrap().as_str()));
            } else {
                try!(writeln!(&mut o, "{}", line));
            }
            line.clear(); // clear to reuse the buffer
        }
        Ok(())
    }


    fn usemod(&mut self, mut o: &mut File, mod_name: &str) -> Result<(), io::Error> {
        let mod_filename0 = format!("src/{}.rs", mod_name);
        let mod_filename = Path::new(&mod_filename0);
        let mod_fd = File::open(mod_filename)
            .expect(&format!("could not open module {:?}", mod_filename));
        let mut mod_reader = BufReader::new(&mod_fd);

        let mod_re = Regex::new(r"^\s*pub mod (.+);$").unwrap();

        let mut line = String::new();

        try!(writeln!(&mut o, "pub mod {} {{", mod_name));

        while mod_reader.read_line(&mut line).unwrap() > 0 {
            line.pop();
            if let Some(cap) = mod_re.captures(&line) {
                try!(self.usemod(o, cap.get(1).unwrap().as_str()));
            } else {
                try!(writeln!(&mut o, "{}", line));
            }
            line.clear(); // clear to reuse the buffer
        }

        try!(writeln!(&mut o, "}}"));

        Ok(())
    }
}
