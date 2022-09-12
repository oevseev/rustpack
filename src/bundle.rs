use std::collections::HashMap;

use camino::{Utf8PathBuf, Utf8Path};

use crate::manifest::CratePaths;

#[derive(Debug)]
pub(crate) struct Context<'a> {
    root_manifest_dir: Utf8PathBuf,
    out_dir: Utf8PathBuf,
    crate_paths: &'a HashMap<String, CratePaths>,
    root_src_path: Utf8PathBuf,
    out_path: Utf8PathBuf,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        manifest_dir: &Utf8Path,
        out_dir: &Utf8Path,
        crate_paths: &'a HashMap<String, CratePaths>,
        src_path: &Utf8Path,
        out_path: &Utf8Path
    ) -> Context<'a> {
        Context {
            root_manifest_dir: manifest_dir.to_path_buf(),
            out_dir: out_dir.to_path_buf(),
            crate_paths,
            root_src_path: src_path.to_path_buf(),
            out_path: out_path.to_path_buf(),
        }
    }
}

pub(crate) fn bundle(ctx: &mut Context) {
    println!("{:#?}", ctx);
}