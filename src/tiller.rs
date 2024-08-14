use std::{
    collections::{HashMap, HashSet},
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
        Self {
            matter: Matter::<YAML>::new(),
            md_options: Options::default(),
            md_adapter: SyntectAdapterBuilder::new()
                .theme("Solarized (light)")
                .build(),
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
                .parse_with_struct::<Meta>(&raw_til)
                .ok_or(anyhow!("couldn't parse front matter"))?;

            let mut plugins = Plugins::default();

            plugins.render.codefence_syntax_highlighter = Some(&self.md_adapter);

            let content =
                markdown_to_html_with_plugins(&parsed.content, &self.md_options, &plugins);

            tils.push(TIL {
                meta: parsed.data,
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
    pub(crate) tags: HashSet<String>,
    pub(crate) date: String,
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

    pub(crate) fn by_tag(&self) -> HashMap<&str, Vec<&TIL>> {
        let mut tils_by_tag = HashMap::new();
        for tag in self.tags() {
            let tils = self
                .0
                .iter()
                .filter(|til| til.meta.tags.contains(tag))
                .collect::<Vec<_>>();

            tils_by_tag.insert(tag, tils);
        }

        tils_by_tag
    }

    pub(crate) fn tag_counts(&self) -> HashMap<&str, usize> {
        let mut counts = HashMap::new();
        for (tag, tils) in self.by_tag() {
            counts.insert(tag, tils.len());
        }

        counts
    }
}
