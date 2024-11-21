use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) base_url: String,
    pub(crate) mastodon: Option<String>,
}
