use std::env;

use camino::Utf8Path;

use crate::bundle::bundle_all;

pub fn build() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR should be set");

    bundle_all(Utf8Path::new(&manifest_dir), Utf8Path::new(&out_dir))
}
