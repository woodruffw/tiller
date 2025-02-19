use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{Context, Result};
use comrak::{markdown_to_html, Options};
use handlebars::{handlebars_helper, Handlebars};
use rss::{CategoryBuilder, ChannelBuilder, ItemBuilder};
use rust_embed::Embed;
use serde::Serialize;
use syntect::highlighting::ThemeSet;

use crate::{
    config::Config,
    tiller::{Meta, TILs, TIL},
};

#[derive(Embed)]
#[folder = "assets/templates"]
#[include = "*.hbs"]
struct Templates;

#[derive(Embed)]
#[folder = "assets/partials"]
#[include = "*.hbs"]
struct Partials;

#[derive(Embed)]
#[folder = "assets/static"]
struct Static;

#[derive(Serialize)]
struct Index<'a> {
    config: &'a Config,
    index_fragment: Option<&'a str>,
    tag_counts: BTreeMap<&'a str, usize>,
    recent: Vec<&'a TIL>,
}

#[derive(Serialize)]
struct Category<'a> {
    config: &'a Config,
    tag: &'a str,
    tils: Vec<&'a TIL>,
}

#[derive(Serialize)]
struct TILPost<'a> {
    config: &'a Config,
    meta: &'a Meta,
    content: &'a str,
}

impl<'a> TILPost<'a> {
    fn new(config: &'a Config, til: &'a TIL) -> Self {
        Self {
            config,
            meta: &til.meta,
            content: &til.content,
        }
    }
}

pub(crate) struct Renderer {
    outdir: PathBuf,
    config: Config,
    index: Option<String>,
    tils: TILs,
    hbs: Handlebars<'static>,
}

impl Renderer {
    pub(crate) fn new(
        outdir: PathBuf,
        config: Config,
        index: Option<String>,
        tils: TILs,
    ) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        hbs.register_embed_templates::<Templates>()?;
        // Partials intentionally have .hbs removed since it's visual clutter
        // when referenced within the templates themselves.
        hbs.register_embed_templates_with_extension::<Partials>(".hbs")?;

        // Inject some useful helpers.
        handlebars_helper!(slugify: |x: String| slug::slugify(x));
        hbs.register_helper("slugify", Box::new(slugify));

        let mut options = Options::default();
        options.extension.footnotes = true;
        options.extension.strikethrough = true;
        options.extension.superscript = true;
        options.extension.underline = true;
        options.extension.table = true;
        let index = index.map(|i| markdown_to_html(&i, &options));

        Ok(Self {
            outdir,
            config,
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
            config: &self.config,
            index_fragment: self.index.as_deref(),
            tag_counts: self.tils.tag_counts(),
            recent: self.tils.by_age().take(20).collect(),
        };
        let index_html = self.hbs.render("index.hbs", &index)?;
        std::fs::write(self.outdir.join("index.html"), index_html)?;

        // Category pages.
        let categories_dir = self.outdir.join("category");
        std::fs::create_dir_all(&categories_dir)
            .with_context(|| "failed to create categories dir")?;
        for (tag, tils) in self.tils.by_tag() {
            let category_dir = categories_dir.join(tag);
            std::fs::create_dir_all(&category_dir)
                .with_context(|| "failed to create individual category dir")?;
            let category = Category {
                config: &self.config,
                tag,
                tils,
            };
            let category_html = self.hbs.render("category.hbs", &category)?;
            std::fs::write(category_dir.join("index.html"), category_html)?;
        }

        // Individual TILs.
        let posts_dir = self.outdir.join("post");
        std::fs::create_dir_all(&posts_dir).with_context(|| "failed to create posts dir")?;
        for til in self.tils.0.iter() {
            let slug = slug::slugify(&til.meta.title);
            let post_dir = posts_dir.join(&slug);
            std::fs::create_dir_all(&post_dir)
                .with_context(|| "failed to create individual post dir")?;
            let til_html = self
                .hbs
                .render("til.hbs", &TILPost::new(&self.config, til))?;
            std::fs::write(post_dir.join("index.html"), &til_html)?;
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
            .link(&self.config.base_url)
            .items(items)
            .build();
        std::fs::write(self.outdir.join("feed.rss"), channel.to_string())?;

        Ok(())
    }
}
