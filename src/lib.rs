// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

#![warn(rust_2018_idioms)]

//! Use this library in your build.rs to create a single file with all
//! the crate's source code.
//!
//! That's useful for programming exercise sites that take a single
//! source file.

pub mod bundler;
pub use bundler::*;
