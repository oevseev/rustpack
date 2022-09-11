use std::collections::HashMap;

use camino::{Utf8PathBuf, Utf8Path};
use cargo_metadata::{MetadataCommand, Package, Target};

#[derive(Debug)]
pub(crate) struct CratePaths {
    manifest_dir: Utf8PathBuf,
    src_path: Utf8PathBuf,
}

fn get_pkg_manifest_dir(pkg: &Package) -> Utf8PathBuf {
    pkg.manifest_path.parent().unwrap().to_owned()
}

fn get_lib_crate_target(pkg: &Package) -> Option<&Target> {
    pkg.targets.iter().find(|target| target.kind.iter().any(|k| k == "lib"))
}

fn get_lib_crate_name(target: &Target) -> String {
    // See https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-name-field for explanation
    target.name.replace("-", "_")
}

fn get_lib_crate_relative_src_path(pkg: &Package, target: &Target) -> Utf8PathBuf {
    target.src_path
        .strip_prefix(pkg.manifest_path.parent().unwrap())
        .unwrap()
        .to_owned()
}

pub(crate) fn get_crate_paths(manifest_dir: &Utf8Path) -> HashMap<String, CratePaths> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_dir.join("Cargo.toml"))
        .exec()
        .unwrap();
        
    metadata.packages.iter()
        // Use filter_map because are interested only in library crates
        .filter_map(|pkg| get_lib_crate_target(pkg)
            .map(|target| (get_lib_crate_name(target), CratePaths {
                manifest_dir: get_pkg_manifest_dir(pkg),
                src_path: get_lib_crate_relative_src_path(pkg, target),
            })))
        .collect()
}
