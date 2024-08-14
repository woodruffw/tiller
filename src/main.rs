use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Parser;

mod render;
mod tiller;

/// Yet another TIL tracker.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The directory to run from.
    /// Defaults to $CWD/til.
    #[arg(short, long)]
    indir: Option<PathBuf>,

    /// The directory to render into.
    /// Defaults to $CWD/site.
    #[arg(short, long)]
    outdir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cwd = std::env::current_dir()?;
    let tildir = match args.indir {
        Some(indir) => indir,
        None => [cwd.clone(), "til".into()].iter().collect(),
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

    let renderer = render::Renderer::new(outdir, tils)?;
    renderer.render()
}
