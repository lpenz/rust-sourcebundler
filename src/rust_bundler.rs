// This is essentially https://github.com/slava-sh/rust-bundler/blob/master/src/lib.rs
// with more structs and functions exported

//! See [README.md](https://github.com/slava-sh/rust-bundler/blob/master/README.md)
extern crate cargo_metadata;
extern crate quote;
extern crate syn;

use std::fs::File;
use std::io::Read;
use std::mem;
use std::path::Path;

use quote::ToTokens;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;

/// Creates a single-source-file version of a Cargo package.
pub fn bundle<P: AsRef<Path>>(package_path: P) -> String {
    let manifest_path = package_path.as_ref().join("Cargo.toml");
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
        .expect("failed to obtain cargo metadata");
    let targets = &metadata.packages[0].targets;
    let bins: Vec<_> = targets.iter().filter(|t| target_is(t, "bin")).collect();
    assert!(!bins.is_empty(), "no binary target found");
    assert!(bins.len() == 1, "multiple binary targets not supported");
    let bin = bins[0];
    let libs: Vec<_> = targets.iter().filter(|t| target_is(t, "lib")).collect();
    assert!(libs.len() <= 1, "multiple library targets not supported");
    let lib = libs.get(0).unwrap_or(&bin);
    let base_path = Path::new(&lib.src_path)
        .parent()
        .expect("lib.src_path has no parent");
    let crate_name = &lib.name;
    eprintln!("expanding binary {}", bin.src_path);
    let code = read_file(Path::new(&bin.src_path)).expect("failed to read binary target source");
    let mut file = syn::parse_file(&code).expect("failed to parse binary target source");
    Expander {
        base_path,
        crate_name,
    }
    .visit_file_mut(&mut file);
    let code = file.into_token_stream().to_string();
    code
}

fn target_is(target: &cargo_metadata::Target, target_kind: &str) -> bool {
    target.kind.iter().any(|kind| kind == target_kind)
}

pub struct Expander<'a> {
    pub base_path: &'a Path,
    pub crate_name: &'a str,
}

impl<'a> Expander<'a> {
    fn expand_items(&self, items: &mut Vec<syn::Item>) {
        self.expand_extern_crate(items);
        self.expand_use_path(items);
    }

    fn expand_extern_crate(&self, items: &mut Vec<syn::Item>) {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if is_extern_crate(&item, self.crate_name) {
                eprintln!(
                    "expanding crate {} in {}",
                    self.crate_name,
                    self.base_path.to_str().unwrap()
                );
                let code =
                    read_file(&self.base_path.join("lib.rs")).expect("failed to read lib.rs");
                let lib = syn::parse_file(&code).expect("failed to parse lib.rs");
                new_items.extend(lib.items);
            } else {
                new_items.push(item);
            }
        }
        *items = new_items;
    }

    fn expand_use_path(&self, items: &mut Vec<syn::Item>) {
        let mut new_items = vec![];
        for item in items.drain(..) {
            if !is_use_path(&item, self.crate_name) {
                new_items.push(item);
            }
        }
        *items = new_items;
    }

    fn expand_mods(&self, item: &mut syn::ItemMod) {
        if item.content.is_some() {
            return;
        }
        let name = item.ident.to_string();
        let other_base_path = self.base_path.join(&name);
        let (base_path, code) = vec![
            (self.base_path, format!("{}.rs", name)),
            (&other_base_path, String::from("mod.rs")),
        ]
        .into_iter()
        .flat_map(|(base_path, file_name)| {
            read_file(&base_path.join(file_name)).map(|code| (base_path, code))
        })
        .next()
        .expect("mod not found");
        eprintln!("expanding mod {} in {}", name, base_path.to_str().unwrap());
        let mut file = syn::parse_file(&code).expect("failed to parse file");
        Expander {
            base_path,
            crate_name: self.crate_name,
        }
        .visit_file_mut(&mut file);
        item.content = Some((Default::default(), file.items));
    }

    fn expand_crate_path(&mut self, path: &mut syn::Path) {
        if path_starts_with(path, self.crate_name) {
            let new_segments = mem::replace(&mut path.segments, Punctuated::new())
                .into_pairs()
                .skip(1)
                .collect();
            path.segments = new_segments;
        }
    }
}

impl<'a> VisitMut for Expander<'a> {
    fn visit_file_mut(&mut self, file: &mut syn::File) {
        for it in &mut file.attrs {
            self.visit_attribute_mut(it)
        }
        self.expand_items(&mut file.items);
        for it in &mut file.items {
            self.visit_item_mut(it)
        }
    }

    fn visit_item_mod_mut(&mut self, item: &mut syn::ItemMod) {
        for it in &mut item.attrs {
            self.visit_attribute_mut(it)
        }
        self.visit_visibility_mut(&mut item.vis);
        self.visit_ident_mut(&mut item.ident);
        self.expand_mods(item);
        if let Some(ref mut it) = item.content {
            for it in &mut (it).1 {
                self.visit_item_mut(it);
            }
        }
    }

    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        self.expand_crate_path(path);
        for mut el in Punctuated::pairs_mut(&mut path.segments) {
            let it = el.value_mut();
            self.visit_path_segment_mut(it)
        }
    }
}

fn is_extern_crate(item: &syn::Item, crate_name: &str) -> bool {
    if let syn::Item::ExternCrate(ref item) = *item {
        if item.ident == crate_name {
            return true;
        }
    }
    false
}

fn path_starts_with(path: &syn::Path, segment: &str) -> bool {
    if let Some(el) = path.segments.first() {
        if el.ident == segment {
            return true;
        }
    }
    false
}

fn is_use_path(item: &syn::Item, first_segment: &str) -> bool {
    if let syn::Item::Use(ref item) = *item {
        if let syn::UseTree::Path(ref path) = item.tree {
            if path.ident == first_segment {
                return true;
            }
        }
    }
    false
}

pub fn read_file(path: &Path) -> Option<String> {
    let mut buf = String::new();
    File::open(path).ok()?.read_to_string(&mut buf).ok()?;
    Some(buf)
}
