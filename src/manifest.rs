use std::collections::HashMap;

use camino::{Utf8PathBuf, Utf8Path};
use cargo_metadata::{MetadataCommand, Package, Target, Metadata};

#[derive(Debug)]
pub(crate) struct CratePaths {
    manifest_dir: Utf8PathBuf,
    src_path: Utf8PathBuf,
}

#[derive(Debug)]
pub(crate) struct Paths {
    crate_paths: HashMap<String, CratePaths>,
    target_paths: HashMap<String, Utf8PathBuf>,
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

fn get_relative_src_path(pkg: &Package, target: &Target) -> Utf8PathBuf {
    target.src_path
        .strip_prefix(pkg.manifest_path.parent().unwrap())
        .unwrap()
        .to_owned()
}

fn get_crate_paths(metadata: &Metadata) -> HashMap<String, CratePaths> {    
    metadata.packages.iter()
        // Use filter_map because are interested only in library crates
        .filter_map(|pkg| get_lib_crate_target(pkg)
            .map(|target| (get_lib_crate_name(target), CratePaths {
                manifest_dir: get_pkg_manifest_dir(pkg),
                src_path: get_relative_src_path(pkg, target),
            })))
        .collect()
}

fn get_target_paths(metadata: &Metadata) -> HashMap<String, Utf8PathBuf> {
    let root_pkg = metadata.root_package().expect("failed to determine root package");

    root_pkg.targets.iter()
        .map(|target| (target.name.clone(), get_relative_src_path(root_pkg, target)))
        .collect()
}

pub(crate) fn process_manifest(manifest_dir: &Utf8Path) -> Paths {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_dir.join("Cargo.toml"))
        .exec()
        .unwrap();

    Paths {
        crate_paths: get_crate_paths(&metadata),
        target_paths: get_target_paths(&metadata),
    }
}
