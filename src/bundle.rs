use camino::Utf8Path;

use crate::manifest::process_manifest;

pub fn bundle_bin(manifest_dir: &Utf8Path, out_dir: &Utf8Path, bin: Option<&str>) {
    println!("{:#?}", process_manifest(manifest_dir))
}

pub fn bundle_all(manifest_dir: &Utf8Path, out_dir: &Utf8Path) {
    println!("{:#?}", process_manifest(manifest_dir))
}
