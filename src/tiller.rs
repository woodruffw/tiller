use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{anyhow, Result};
use comrak::{
    markdown_to_html_with_plugins, options,
    plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder},
    Options,
};
use gray_matter::{engine::YAML, Matter};
use jiff::civil::Date;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

            let parsed = self.matter.parse::<Meta>(&raw_til).map_err(|err| {
                anyhow!(
                    "couldn't parse front matter for {}: {err}",
                    til_file.display()
                )
            })?;

            let mut plugins = options::Plugins::default();

            plugins.render.codefence_syntax_highlighter = Some(&self.md_adapter);

            let content =
                markdown_to_html_with_plugins(&parsed.content, &self.md_options, &plugins);

            tils.push(TIL {
                meta: parsed.data.ok_or(anyhow!("missing front matter data"))?,
                content,
            })
        }

        // TODO: impl Ord for TIL
        tils.sort_unstable_by(|a, b| a.meta.date.cmp(&b.meta.date));

        Ok(TILs(tils))
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub(crate) struct Meta {
    pub(crate) title: String,
    pub(crate) tags: BTreeSet<String>,
    #[serde(
        deserialize_with = "deserialize_til_date",
        serialize_with = "serialize_til_date"
    )]
    pub(crate) date: Date,
    pub(crate) origin: Option<String>,
}

fn deserialize_til_date<'de, D>(deserializer: D) -> std::result::Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    parse_til_date(&raw).map_err(serde::de::Error::custom)
}

fn serialize_til_date<S>(date: &Date, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.to_string())
}

fn parse_til_date(raw: &str) -> std::result::Result<Date, String> {
    let is_yyyy_mm_dd = raw.len() == 10
        && raw.as_bytes()[4] == b'-'
        && raw.as_bytes()[7] == b'-'
        && raw
            .bytes()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit());

    if !is_yyyy_mm_dd {
        return Err(format!("date must be in YYYY-MM-DD format, got {raw:?}"));
    }

    raw.parse()
        .map_err(|err| format!("invalid YYYY-MM-DD date {raw:?}: {err}"))
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
        sorted.sort_by(|a, b| a.meta.date.cmp(&b.meta.date));
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

            tils.sort_by(|a, b| a.meta.date.cmp(&b.meta.date));
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

    pub(crate) fn by_date(&self) -> BTreeMap<String, Vec<&TIL>> {
        let mut tils_by_date = BTreeMap::new();
        for til in &self.0 {
            let key = format!("{:04}-{:02}", til.meta.date.year(), til.meta.date.month());
            tils_by_date.entry(key).or_insert_with(Vec::new).push(til);
        }

        for tils in tils_by_date.values_mut() {
            tils.sort_by(|a, b| a.meta.date.cmp(&b.meta.date));
            tils.reverse();
        }

        tils_by_date
    }
}
