use std::{collections::HashMap, fs::File, io::Read};

use camino::{Utf8PathBuf, Utf8Path};
use quote::{quote, ToTokens};
use syn::{ItemMod, parse_quote, Ident, Item, UseTree, visit_mut::{VisitMut, self}, punctuated::Punctuated, PathSegment, Token, Meta, Lit};

use crate::manifest::CratePaths;

#[derive(Debug)]
pub(crate) struct Bundler<'a> {
    root_manifest_dir: Utf8PathBuf,
    out_dir: Utf8PathBuf,
    crate_paths: &'a HashMap<String, CratePaths>,
    root_src_path: Utf8PathBuf,
    out_path: Utf8PathBuf,
    crate_modules: HashMap<String, ItemMod>,
    out_file: syn::File,
}

impl<'a> Bundler<'a> {
    pub(crate) fn new(
        manifest_dir: &Utf8Path,
        out_dir: &Utf8Path,
        crate_paths: &'a HashMap<String, CratePaths>,
        src_path: &Utf8Path,
        out_path: &Utf8Path
    ) -> Bundler<'a> {
        Bundler {
            root_manifest_dir: manifest_dir.to_path_buf(),
            out_dir: out_dir.to_path_buf(),
            crate_paths,
            root_src_path: src_path.to_path_buf(),
            out_path: out_path.to_path_buf(),
            crate_modules: HashMap::new(),
            out_file: syn::File{
                shebang: None,
                attrs: Vec::new(),
                items: Vec::new(),
            },
        }
    }

    pub(crate) fn bundle(&mut self) {
        self.consolidate();

        // TODO: Output result to file
        println!("{:#?}", self.out_file.clone().into_token_stream().to_string());
    }

    fn consolidate(&mut self) {
        let src_path = self.root_src_path.clone();
        
        self.out_file = parse_file(&src_path);
        Visitor::new(&src_path, "").visit_file_mut(&mut self.out_file)
    }

    fn get_crate_module(&mut self, crate_name: &str) -> &mut ItemMod {
        self.crate_modules.entry(crate_name.to_owned()).or_insert({
            let module_ident: Ident = syn::parse_str(crate_name).expect("crate name must be a valid identifier");
            parse_quote! {
                pub mod #module_ident {}
            }
        })
    }
}

struct Visitor {
    crate_name: Option<Ident>,
    extern_crates: Vec<String>,
    src_paths: Vec<Utf8PathBuf>,
}

impl Visitor {
    fn new(src_path: &Utf8Path, crate_name: &str) -> Visitor {
        let crate_name_ident = if crate_name != "" {
            Some(syn::parse_str(crate_name).expect("crate name must be a valid identifier"))
        } else {
            None
        };

        Visitor{
            crate_name: crate_name_ident,
            extern_crates: Vec::new(),
            src_paths: vec![src_path.to_owned()],
        }
    }

    fn expand_mod_item(&mut self, i: &ItemMod) -> (ItemMod, Utf8PathBuf) {
        let mut expanded_mod = i.clone();

        let path_attr_value = i.attrs.iter()
            .filter_map(|attr| attr.parse_meta().ok())
            .find(|meta| match meta {
                Meta::NameValue(nv) => nv.path.is_ident("path"),
                _ => false,
            })
            .and_then(|meta| match meta {
                Meta::NameValue(nv) => match nv.lit {
                    Lit::Str(s) => Some(s.value()),
                    _ => None,
                },
                _ => None,
            });

        let mod_src_path = if path_attr_value.is_none() {
            let src_dir = self.src_paths.last().unwrap().parent().unwrap();
            let module_name = i.ident.to_string();

            let file_src_path = Utf8PathBuf::from(module_name.clone()).with_extension("rs");
            let absolute_file_src_path = src_dir.join(file_src_path);

            let mod_rs_src_path = Utf8PathBuf::from(module_name.clone()).join("mod.rs");            
            let absolute_mod_rs_src_path = src_dir.join(mod_rs_src_path);

            let src_paths = vec![absolute_file_src_path, absolute_mod_rs_src_path];
            src_paths.iter()
                .find(|p| p.exists())
                .expect("module file should exist")
                .to_owned()
        } else {
            Utf8PathBuf::from(path_attr_value.unwrap())
        };

        let module_file = parse_file(&mod_src_path);
        expanded_mod.content = Some((Default::default(), module_file.items));

        (expanded_mod, mod_src_path)
    }
}

impl VisitMut for Visitor {
    fn visit_item_mut(&mut self, i: &mut Item) {
        // Remove any extern crate declarations as they're not needed anymore
        // (but collect their names for future reference)
        if let Item::ExternCrate(item_extern_crate) = i {
            self.extern_crates.push(item_extern_crate.ident.to_string());
            *i = Item::Verbatim(quote! {});
            return
        }

        visit_mut::visit_item_mut(self, i)
    }

    fn visit_item_mod_mut(&mut self, i: &mut ItemMod) {
        if i.content.is_some() {
            return visit_mut::visit_item_mod_mut(self, i)
        }

        let (expanded_mod, src_path) = self.expand_mod_item(i);
        *i = expanded_mod;

        self.src_paths.push(src_path);
        visit_mut::visit_item_mod_mut(self, i);
        self.src_paths.pop();
    }

    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.segments[0].ident == "crate" {
            if self.crate_name.is_some() {
                // Replace "crate::" prefix with corresponding module name prefix
                i.segments[0].ident = self.crate_name.clone().unwrap()
            } else {
                // Strip "crate::" prefix
                let new_path: Punctuated<PathSegment, Token![::]> = i.segments.clone().into_iter().skip(1).collect();
                i.segments = new_path
            }
        }

        visit_mut::visit_path_mut(self, i)
    }

    fn visit_use_tree_mut(&mut self, i: &mut UseTree) {
        if let UseTree::Path(ref mut path) = i {
            if path.ident != "crate" {
                return visit_mut::visit_use_tree_mut(self, i)
            }
            if self.crate_name.is_some() {
                // Replace "crate::" prefix with corresponding module name prefix
                path.ident = self.crate_name.clone().unwrap()
            } else {
                // Strip "crate::" prefix
                *i = *path.tree.clone()
            }
        }

        visit_mut::visit_use_tree_mut(self, i)
    }
}

fn parse_file(src_path: &Utf8Path) -> syn::File {
    let mut file = File::open(&src_path).expect("file should be available for opening");

    let mut src = String::new();
    file.read_to_string(&mut src).expect("file should be available for reading");

    syn::parse_file(&src).expect("file should be a valid Rust source file")
}