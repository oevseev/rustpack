use std::env;

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;

use rustpack::bundle_bin;

#[derive(Parser)]
struct Args {
    /// Path to manifest directory (i.e. Rust package)
    #[clap(value_parser)]
    manifest_dir: String,

    /// Output directory (optional)
    #[clap(long, value_parser)]
    out_dir: Option<String>,

    /// Binary to bundle (optional)
    #[clap(long, value_parser)]
    bin: Option<String>,
}

fn main() {
    let args = Args::parse();
    let out_dir = match args.out_dir {
        Some(s) => Utf8PathBuf::from(s),
        None => {
            let current_dir = env::current_dir().expect("error getting current dir");
            Utf8PathBuf::from_path_buf(current_dir).expect("current dir is not a valid UTF-8 path")
        },
    };
    
    bundle_bin(Utf8Path::new(&args.manifest_dir), out_dir.as_path(), args.bin.as_deref())
}
