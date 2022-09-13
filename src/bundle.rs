use core::fmt;
use std::{collections::HashMap, fs::File, io::{Read, Write}, error::Error, fmt::Display};

use camino::{Utf8PathBuf, Utf8Path};
use quote::{quote, ToTokens};
use syn::{ItemMod, parse_quote, Ident, Item, UseTree, visit_mut::{VisitMut, self}, PathSegment, Meta, Lit, PathArguments, UsePath};

use crate::manifest::CratePaths;

#[derive(Debug)]
pub(crate) struct BundleError {
    msg: String,
}

impl BundleError {
    fn new(msg: &str) -> BundleError {
        BundleError { msg: msg.to_owned() }
    }
}

impl Display for BundleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for BundleError {}

#[derive(Debug)]
pub(crate) struct Bundler<'a> {
    crate_paths: &'a HashMap<String, CratePaths>,
    root_src_path: Utf8PathBuf,
    out_path: Utf8PathBuf,
    crate_modules: HashMap<String, ItemMod>,
    out_file: syn::File,
}

impl<'a> Bundler<'a> {
    pub(crate) fn new(
        crate_paths: &'a HashMap<String, CratePaths>,
        src_path: &Utf8Path,
        out_path: &Utf8Path
    ) -> Bundler<'a> {
        Bundler {
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

        let mut file = File::create(&self.out_path).expect("out file should be opened to proceed");
        file.write(self.out_file.clone().into_token_stream().to_string().as_bytes()).expect("out file should be available for write");
    }

    fn consolidate(&mut self) {
        let src_path = self.root_src_path.clone();

        self.out_file = parse_file(&src_path).expect("root source must be parsed correctly to proceed");
        Visitor::new(&src_path, "").visit_file_mut(&mut self.out_file);

        for (crate_name, crate_paths) in self.crate_paths {
            let crate_src_path = crate_paths.manifest_dir.join(crate_paths.src_path.clone());
            let crate_file = parse_file(&crate_src_path).expect("crate source must be parsed correctly to proceed");
            let ref mut crate_module = self.get_crate_module(crate_name);

            crate_module.attrs = crate_file.attrs;
            crate_module.content = Some((Default::default(), crate_file.items));

            let mut visitor = Visitor::new(&crate_src_path, crate_name);

            // Call overridden function instead of Visitor's own to avoid applying rules to the root module
            visit_mut::visit_item_mod_mut(&mut visitor, crate_module);
        }

        for (_, crate_module) in self.crate_modules.drain() {
            self.out_file.items.push(Item::Mod(crate_module));
        }
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

    fn expand_mod_item(&mut self, i: &ItemMod) -> Result<(ItemMod, Utf8PathBuf), Box<dyn Error>> {
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

        let src_dir = self.src_paths.last().unwrap().parent().unwrap();

        let mod_src_path = if path_attr_value.is_none() {
            let module_name = i.ident.to_string();

            let file_src_path = Utf8PathBuf::from(module_name.clone()).with_extension("rs");
            let absolute_file_src_path = src_dir.join(file_src_path);

            let mod_rs_src_path = Utf8PathBuf::from(module_name.clone()).join("mod.rs");            
            let absolute_mod_rs_src_path = src_dir.join(mod_rs_src_path);

            let src_paths = vec![absolute_file_src_path, absolute_mod_rs_src_path];
            src_paths.iter().find(|p| p.exists()).and_then(|p| Some(p.to_owned()))
        } else {
            Some(src_dir.clone().join(path_attr_value.unwrap()).canonicalize_utf8()?)
        };

        if let Some(p) = mod_src_path {
            if !p.exists() {
                return Err(Box::new(BundleError::new("module file does not exist")))
            }

            let module_file = parse_file(&p)?;
            expanded_mod.content = Some((Default::default(), module_file.items));

            Ok((expanded_mod, p.to_owned()))
        } else {
            Err(Box::new(BundleError::new("module file does not exist")))
        }
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
            let src_path = self.src_paths.last().unwrap().parent().unwrap();
            let pseudo_src_path = src_path.join(i.ident.to_string()).join("mod.rs");

            self.src_paths.push(pseudo_src_path);
            visit_mut::visit_item_mod_mut(self, i);
            self.src_paths.pop();

            return
        }

        if let Ok((expanded_mod, src_path)) = self.expand_mod_item(i) {
            *i = expanded_mod;

            self.src_paths.push(src_path);
            visit_mut::visit_item_mod_mut(self, i);
            self.src_paths.pop();
        } else {
            eprintln!("warning: {}: could not expand {}", self.src_paths.last().unwrap(), i.ident.to_string());
        }
    }

    fn visit_visibility_mut(&mut self, _i: &mut syn::Visibility) {
        // No-op: avoid rewriting paths in visibility
        return
    }

    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        if i.segments[0].ident == "crate" {
            if self.crate_name.is_some() {
                // Append module name after "crate::"
                i.segments.insert(1, PathSegment{
                    ident: self.crate_name.clone().unwrap(),
                    arguments: PathArguments::None,
                })
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
                let ref old_tree = path.tree;
                let new_tree = Box::new(UseTree::Path(UsePath{
                    ident: self.crate_name.clone().unwrap(),
                    colon2_token: Default::default(),
                    tree: old_tree.to_owned(),
                }));
                path.tree = new_tree;
            }
        }

        visit_mut::visit_use_tree_mut(self, i)
    }
}

fn parse_file(src_path: &Utf8Path) -> Result<syn::File, Box<dyn Error>> {
    eprintln!("{}", src_path);

    let mut file = File::open(&src_path)?;
    let mut src = String::new();
    file.read_to_string(&mut src)?;

    Ok(syn::parse_file(&src)?)
}
