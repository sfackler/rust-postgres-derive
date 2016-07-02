extern crate postgres_derive_codegen;

use std::env;
use std::path::Path;

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let src = Path::new("src/types.rs.in");
    let dst = Path::new(&out_dir).join("types.rs");

    postgres_derive_codegen::expand(src, dst).unwrap();
}
