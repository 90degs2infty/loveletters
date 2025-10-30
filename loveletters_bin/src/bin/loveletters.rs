//! loveletters commandline interface.

use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use walkdir::{DirEntry, WalkDir};

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

fn is_post_frontmatter(entry: &DirEntry) -> bool {
    entry.file_type().is_file()
        && entry
            .file_name()
            .to_str()
            .map(|name| name == "post.toml")
            .unwrap_or(false)
}

/// A single post's frontmatter.
#[derive(Deserialize, Debug)]
struct PostFrontmatter {
    /// This post's title.
    title: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let posts_dir = args
        .input_dir
        .join("posts")
        .canonicalize()
        .expect("Could not resolve specified input directory");

    for post in WalkDir::new(&posts_dir)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_entry(is_post_frontmatter)
    {
        let post = post.expect(&format!(
            "Failed to access entry. Are you sure {} exists?",
            posts_dir.display()
        ));
        let frontmatter: String = fs::read_to_string(post.path()).expect(&format!(
            "Failed to open frontmatter file at '{}'",
            post.path().display()
        ));
        let frontmatter: PostFrontmatter = toml::from_str(&frontmatter).expect(&format!(
            "Failed to parse frontmatter from file at '{}'",
            post.path().display()
        ));
        println!("{:?}", frontmatter);
    }

    Ok(())
}
