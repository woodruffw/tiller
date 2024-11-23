use std::{fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use config::Config;

mod config;
mod render;
mod tiller;

/// Yet another TIL tracker.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The directory to render from. Must contain a `tils` subdirectory.
    /// Defaults to $CWD/tils.
    #[arg(short, long)]
    indir: Option<PathBuf>,

    /// The directory to render into.
    /// Defaults to $CWD/site.
    #[arg(short, long)]
    outdir: Option<PathBuf>,

    /// Generates in 'dev' mode, meaning suitable for use with a local
    /// development HTTP server.
    #[arg(long)]
    dev: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let cwd = std::env::current_dir()?;

    let indir = match args.indir {
        Some(indir) => indir,
        None => cwd.clone(),
    };

    let tildir = indir.join("tils");

    let index = indir.join("_index.md");
    let index = match index.is_file() {
        true => Some(fs::read_to_string(index)?),
        false => None,
    };

    let mut config = toml::from_str::<Config>(
        &fs::read_to_string(indir.join("tiller.toml"))
            .with_context(|| "could not load config file")?,
    )?;

    if args.dev {
        config.base_url = "/".into();
    } else {
        // All URL joining assumes a terminating `/`.
        if !config.base_url.ends_with('/') {
            config.base_url.push('/');
        }
    }

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

    let renderer = render::Renderer::new(outdir, config, index, tils)?;
    renderer.render()
}
