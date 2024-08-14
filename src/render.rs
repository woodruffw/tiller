use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use handlebars::{handlebars_helper, Handlebars};
use rust_embed::Embed;
use serde::Serialize;

use crate::tiller::{TILs, TIL};

#[derive(Embed)]
#[folder = "assets/templates"]
#[include = "*.hbs"]
struct Templates;

#[derive(Embed)]
#[folder = "assets/static"]
struct Static;

#[derive(Serialize)]
struct Index<'a> {
    tag_counts: HashMap<&'a str, usize>,
    recent: Vec<&'a TIL>,
}

#[derive(Serialize)]
struct Category<'a> {
    tag: &'a str,
    tils: Vec<&'a TIL>,
}

pub(crate) struct Renderer {
    outdir: PathBuf,
    tils: TILs,
    hbs: Handlebars<'static>,
}

impl Renderer {
    pub(crate) fn new(outdir: PathBuf, tils: TILs) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);
        hbs.register_embed_templates::<Templates>()?;

        // Inject some useful helpers.
        handlebars_helper!(slugify: |x: String| slug::slugify(x));
        hbs.register_helper("slugify", Box::new(slugify));

        Ok(Self { outdir, tils, hbs })
    }

    pub(crate) fn render(&self) -> Result<()> {
        // Static assets (CSS, JS).
        let style = Static::get("style.css").unwrap();
        std::fs::write(self.outdir.join("style.css"), style.data)?;

        // Index page.
        let index = Index {
            tag_counts: self.tils.tag_counts(),
            recent: self.tils.by_age().take(20).collect(),
        };
        let index_html = self.hbs.render("index.hbs", &index)?;
        std::fs::write(self.outdir.join("index.html"), index_html)?;

        // Category pages.
        std::fs::create_dir_all(self.outdir.join("category"))?;
        for (tag, tils) in self.tils.by_tag() {
            let category = Category { tag, tils };
            let category_html = self.hbs.render("category.hbs", &category)?;
            std::fs::write(self.outdir.join("category").join(tag), category_html)?;
        }

        // Individual TILs.
        std::fs::create_dir_all(self.outdir.join("til"))?;
        for til in self.tils.0.iter() {
            let til_html = self.hbs.render("til.hbs", &til)?;
            std::fs::write(
                self.outdir.join("til").join(slug::slugify(&til.meta.title)),
                &til_html,
            )?;
        }

        Ok(())
    }
}
