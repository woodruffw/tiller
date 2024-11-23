use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) base_url: String,
    pub(crate) mastodon: Option<String>,
    #[serde(default)]
    pub(crate) top_links: Vec<Link>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Link {
    title: String,
    url: String,
}
