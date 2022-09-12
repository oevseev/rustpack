use std::{collections::HashMap, env};

use camino::{Utf8Path, Utf8PathBuf};

use crate::{manifest::{process_manifest, CratePaths}, bundle::{bundle, Context}};

fn bundle_src(
    manifest_dir: &Utf8Path,
    out_dir: &Utf8Path,
    crate_paths: &HashMap<String, CratePaths>,
    src_path: &Utf8Path, out_path: &Utf8Path
) {    
    let manifest_dir_absolute = manifest_dir
        .canonicalize_utf8()
        .expect("manifest dir must be canonicalized to proceed");
    let out_dir_absolute = out_dir
        .canonicalize_utf8()
        .expect("output dir must be canonicalized to proceed");

    let root_src_path = manifest_dir_absolute.join(src_path);
    let out_path_absolute = out_dir_absolute.join(out_path);

    let mut ctx = Context::new(
        &manifest_dir_absolute, &out_dir_absolute, crate_paths, &root_src_path, &out_path_absolute);

    bundle(&mut ctx)
}

pub fn bundle_bin(manifest_dir: &Utf8Path, out_dir: &Utf8Path, bin: Option<&str>) {
    let paths = process_manifest(manifest_dir);

    let src_path = paths.target_paths
        .get(bin.unwrap_or(""))
        .expect("binary should exist");
    let out_path = Utf8PathBuf::from(bin.unwrap_or("main")).with_extension("rs");

    bundle_src(manifest_dir, out_dir, &paths.crate_paths, src_path, &out_path);
}

pub fn bundle_all(manifest_dir: &Utf8Path, out_dir: &Utf8Path) {
    let paths = process_manifest(manifest_dir);

    for (ref target_name, ref src_path) in paths.target_paths {
        let out_path = if target_name != "" {
            Utf8PathBuf::from(target_name).with_extension("rs")
        } else {
            Utf8PathBuf::from("main.rs")
        };

        bundle_src(manifest_dir, out_dir, &paths.crate_paths, src_path, &out_path)
    }
}

pub fn build() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR should be set");

    bundle_all(Utf8Path::new(&manifest_dir), Utf8Path::new(&out_dir))
}
