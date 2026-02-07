//! loveletters commandline interface.

use clap::Parser;
use loveletters_lib::{error::Result, render_dir};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Make `loveletters` increasingly chatty.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbosity: u8,

    /// Directory to read content from.
    input_dir: PathBuf,

    /// Directory to render output to.
    output_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    render_dir(args.input_dir, args.output_dir)
}
