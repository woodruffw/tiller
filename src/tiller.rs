use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{anyhow, Result};
use comrak::{
    markdown_to_html_with_plugins,
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    Options, Plugins,
};
use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};

/// A Tiller tills TILs.
pub(crate) struct Tiller {
    matter: Matter<YAML>,
    md_options: Options<'static>,
    md_adapter: SyntectAdapter,
}

impl Tiller {
    pub(crate) fn new() -> Self {
        // TODO: Consider making the theme configurable.
        let mut options = Options::default();
        options.extension.footnotes = true;
        options.extension.strikethrough = true;
        options.extension.superscript = true;
        options.extension.underline = true;
        options.extension.table = true;

        Self {
            matter: Matter::<YAML>::new(),
            md_options: options,
            md_adapter: SyntectAdapterBuilder::new().css().build(),
        }
    }

    pub(crate) fn till(&self, tildir: &Path) -> Result<TILs> {
        let mut tils = vec![];
        for til_file in tildir.read_dir()? {
            let til_file = til_file?.path();
            if !til_file.to_string_lossy().ends_with(".md") {
                continue;
            }

            let raw_til = std::fs::read_to_string(&til_file)?;

            let parsed = self
                .matter
                .parse::<Meta>(&raw_til)
                .map_err(|_| anyhow!("couldn't parse front matter"))?;

            let mut plugins = Plugins::default();

            plugins.render.codefence_syntax_highlighter = Some(&self.md_adapter);

            let content =
                markdown_to_html_with_plugins(&parsed.content, &self.md_options, &plugins);

            tils.push(TIL {
                meta: parsed.data.ok_or(anyhow!("missing front matter data"))?,
                content,
            })
        }

        // TODO: impl Ord for TIL
        tils.sort_unstable_by(|a, b| a.meta.date.partial_cmp(&b.meta.date).unwrap());

        Ok(TILs(tils))
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub(crate) struct Meta {
    pub(crate) title: String,
    pub(crate) tags: BTreeSet<String>,
    pub(crate) date: String,
    pub(crate) origin: Option<String>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Serialize)]
/// A single "TIL".
pub(crate) struct TIL {
    pub(crate) meta: Meta,
    pub(crate) content: String,
}

pub(crate) struct TILs(pub(crate) Vec<TIL>);

impl TILs {
    pub(crate) fn by_age(&self) -> impl Iterator<Item = &TIL> {
        let mut sorted = self.0.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.meta.date.partial_cmp(&b.meta.date).unwrap());
        sorted.reverse();

        sorted.into_iter()
    }

    fn tags(&self) -> impl Iterator<Item = &str> {
        self.0
            .iter()
            .flat_map(|til| til.meta.tags.iter())
            .map(|s| s.as_str())
    }

    pub(crate) fn by_tag(&self) -> BTreeMap<&str, Vec<&TIL>> {
        let mut tils_by_tag = BTreeMap::new();
        for tag in self.tags() {
            let mut tils = self
                .0
                .iter()
                .filter(|til| til.meta.tags.contains(tag))
                .collect::<Vec<_>>();

            tils.sort_by(|a, b| a.meta.date.partial_cmp(&b.meta.date).unwrap());
            tils.reverse();

            tils_by_tag.insert(tag, tils);
        }

        tils_by_tag
    }

    pub(crate) fn tag_counts(&self) -> BTreeMap<&str, usize> {
        let mut counts = BTreeMap::new();
        for (tag, tils) in self.by_tag() {
            counts.insert(tag, tils.len());
        }

        counts
    }
}
