use camino::Utf8Path;

use crate::manifest::get_crate_paths;

pub fn bundle_bin(manifest_dir: &Utf8Path, out_dir: &Utf8Path, bin: Option<&str>) {
    println!("{:#?}", get_crate_paths(manifest_dir))
}

pub fn bundle_all(manifest_dir: &Utf8Path, out_dir: &Utf8Path) {
    println!("{:#?}", get_crate_paths(manifest_dir))
}
