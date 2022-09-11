use std::{env, path::Path};

use clap::Parser;

use rustpack::bundle_bin;

#[derive(Parser)]
struct Args {
    /// Path to manifest directory (i.e. Rust package)
    #[clap(value_parser)]
    manifest_dir: String,

    /// Binary to bundle (optional)
    #[clap(long, value_parser)]
    bin: Option<String>,
}

fn main() {
    let args = Args::parse();
    let current_dir = env::current_dir().expect("error getting current directory");
    
    bundle_bin(Path::new(&args.manifest_dir), current_dir.as_path(), args.bin.as_deref())
}
