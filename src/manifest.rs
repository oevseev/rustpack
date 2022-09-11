use std::{path::Path, collections::HashMap};

use cargo_metadata::{MetadataCommand, Package};

fn get_manifest_dir(pkg: &Package) -> String {
    pkg.manifest_path
        .parent()
        .expect("failed to get manifest dir")
        .to_string()
}

fn get_lib_crate_name(pkg: &Package) -> Option<String> {
    pkg.targets.iter()
        .find(|target| target.kind.iter().any(|k| k == "lib"))
        // See https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-name-field for explanation
        .map(|target| target.name.replace("-", "_"))
}

pub(crate) fn get_crate_manifest_dirs(manifest_dir: &Path) -> HashMap<String, String> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_dir.join("Cargo.toml"))
        .exec()
        .unwrap();
        
    metadata.packages.iter()
        // Use filter_map because are interested only in library crates
        .filter_map(|pkg| get_lib_crate_name(pkg).map(|crate_name| (crate_name, get_manifest_dir(pkg))))
        .collect()
}
