use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use comrak::{markdown_to_html, Options};
use handlebars::{handlebars_helper, Handlebars};
use rss::{CategoryBuilder, ChannelBuilder, ItemBuilder};
use rust_embed::Embed;
use serde::Serialize;
use syntect::highlighting::ThemeSet;

use crate::tiller::{Meta, TILs, TIL};

#[derive(Embed)]
#[folder = "assets/templates"]
#[include = "*.hbs"]
struct Templates;

#[derive(Embed)]
#[folder = "assets/static"]
struct Static;

#[derive(Serialize)]
struct Index<'a> {
    base_url: &'a str,
    index_fragment: Option<&'a str>,
    tag_counts: HashMap<&'a str, usize>,
    recent: Vec<&'a TIL>,
}

#[derive(Serialize)]
struct Category<'a> {
    base_url: &'a str,
    tag: &'a str,
    tils: Vec<&'a TIL>,
}

#[derive(Serialize)]
struct TILPost<'a> {
    base_url: &'a str,
    meta: &'a Meta,
    content: &'a str,
}

impl<'a> TILPost<'a> {
    fn new(base_url: &'a str, til: &'a TIL) -> Self {
        Self {
            base_url,
            meta: &til.meta,
            content: &til.content,
        }
    }
}

pub(crate) struct Renderer {
    outdir: PathBuf,
    base_url: String,
    index: Option<String>,
    tils: TILs,
    hbs: Handlebars<'static>,
}

impl Renderer {
    pub(crate) fn new(
        outdir: PathBuf,
        base_url: String,
        index: Option<String>,
        tils: TILs,
    ) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        hbs.register_embed_templates::<Templates>()?;

        // Inject some useful helpers.
        handlebars_helper!(slugify: |x: String| slug::slugify(x));
        hbs.register_helper("slugify", Box::new(slugify));

        let index = index.map(|i| markdown_to_html(&i, &Options::default()));

        Ok(Self {
            outdir,
            base_url,
            index,
            tils,
            hbs,
        })
    }

    pub(crate) fn render(&self) -> Result<()> {
        // Static assets (CSS, JS).
        std::fs::write(
            self.outdir.join("style.css"),
            Static::get("style.css").unwrap().data,
        )?;
        std::fs::write(
            self.outdir.join("index.js"),
            Static::get("index.js").unwrap().data,
        )?;

        std::fs::write(
            self.outdir.join("syntect.css"),
            syntect::html::css_for_theme_with_class_style(
                // TODO: Make this configurable.
                &ThemeSet::load_defaults().themes["Solarized (dark)"],
                syntect::html::ClassStyle::Spaced,
            )?,
        )?;

        // Index page.
        let index = Index {
            base_url: &self.base_url,
            index_fragment: self.index.as_deref(),
            tag_counts: self.tils.tag_counts(),
            recent: self.tils.by_age().take(20).collect(),
        };
        let index_html = self.hbs.render("index.hbs", &index)?;
        std::fs::write(self.outdir.join("index.html"), index_html)?;

        // Category pages.
        let category_dir = self.outdir.join("category");
        std::fs::create_dir_all(category_dir).with_context(|| "failed to create category dir")?;
        for (tag, tils) in self.tils.by_tag() {
            let category = Category {
                base_url: &self.base_url,
                tag,
                tils,
            };
            let category_html = self.hbs.render("category.hbs", &category)?;
            std::fs::write(
                self.outdir
                    .join("category")
                    .join(tag)
                    .with_extension("html"),
                category_html,
            )?;
        }

        // Individual TILs.
        let post_dir = self.outdir.join("post");
        std::fs::create_dir_all(&post_dir).with_context(|| "failed to create post dir")?;
        for til in self.tils.0.iter() {
            let til_html = self
                .hbs
                .render("til.hbs", &TILPost::new(&self.base_url, til))?;
            std::fs::write(
                post_dir
                    .join(slug::slugify(&til.meta.title))
                    .with_extension("html"),
                &til_html,
            )?;
        }

        // RSS feed.
        // TODO: Per-category feeds?
        let mut items = vec![];
        for til in self.tils.by_age().take(20) {
            items.push(
                ItemBuilder::default()
                    .categories(
                        til.meta
                            .tags
                            .iter()
                            .map(|t| CategoryBuilder::default().name(t).build())
                            .collect::<Vec<_>>(),
                    )
                    .title(til.meta.title.clone())
                    // This is technically wrong, since RSS requires RFC 822
                    // timestamps. But I can't be bothered to munge into
                    // such an annoying format.
                    .pub_date(til.meta.date.clone())
                    .content(til.content.clone())
                    .build(),
            );
        }

        let channel = ChannelBuilder::default()
            .title("TILs")
            .link(&self.base_url)
            .items(items)
            .build();
        std::fs::write(self.outdir.join("feed.rss"), channel.to_string())?;

        Ok(())
    }
}
