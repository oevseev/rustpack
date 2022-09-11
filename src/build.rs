use std::{env, path::Path};

use crate::bundle::bundle_all;

pub fn build() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");

    bundle_all(Path::new(&manifest_dir), Path::new(&out_dir))
}
