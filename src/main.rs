use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

mod render;
mod tiller;

/// Yet another TIL tracker.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The directory to render from. Must contain a `tils` subdirectory.
    #[arg(short, long)]
    indir: Option<PathBuf>,

    /// An optional index Markdown fragment to include.
    /// Defaults to {indir}/_index.md, if present.
    #[arg(long)]
    index: Option<PathBuf>,

    /// The base site URL to render links from.
    #[arg(long, default_value = "/")]
    base_url: String,

    /// The directory to render into.
    /// Defaults to $CWD/site.
    #[arg(short, long)]
    outdir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let mut args = Args::parse();

    // All URL joining assumes a terminating `/`.
    if !args.base_url.ends_with('/') {
        args.base_url.push('/');
    }

    let cwd = std::env::current_dir()?;
    let tildir = match args.indir {
        Some(ref indir) => indir.join("tils"),
        None => cwd.join("tils"),
    };

    let index = match args.index {
        Some(index) => Some(std::fs::read_to_string(index)?),
        None => match args.indir {
            Some(indir) if indir.join("_index.md").is_file() => {
                Some(std::fs::read_to_string(indir.join("_index.md"))?)
            }
            _ => None,
        },
    };

    let outdir = match args.outdir {
        Some(outdir) => outdir,
        None => [cwd, "site".into()].iter().collect(),
    };

    if !tildir.is_dir() {
        return Err(anyhow!("expected directory at {tildir:?}"));
    }

    if !outdir.is_dir() {
        std::fs::create_dir(&outdir).with_context(|| "failed to create output directory")?;
    }

    let tiller = tiller::Tiller::new();
    let tils = tiller.till(&tildir)?;

    let renderer = render::Renderer::new(outdir, args.base_url, index, tils)?;
    renderer.render()
}
