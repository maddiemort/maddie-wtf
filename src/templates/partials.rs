use std::{env, option_env};

use maddie_wtf::build_info;
use maud::{html, Markup};

use crate::state::Theme;

pub async fn head(title: Option<&str>, theme: Theme) -> Markup {
    let theme_header = theme.theme_header();
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width,initial-scale=1,height=device-height";

            link rel="icon" href="/static/favicon.svg" type="image/svg+xml";

            link rel="stylesheet" href="/style.css" type="text/css";

            link rel="preload" href="/static/iosevka-regular.woff2" as="font" type="font/woff2" crossorigin;
            link rel="preload" href="/static/IBMPlexSans-Italic.woff2" as="font" type="font/woff2" crossorigin;
            link rel="preload" href="/static/IBMPlexSans-Regular.woff2" as="font" type="font/woff2" crossorigin;
            link rel="preload" href="/static/IBMPlexSans-SemiBold.woff2" as="font" type="font/woff2" crossorigin;
            link rel="preload" href="/static/IBMPlexSans-SemiBoldItalic.woff2" as="font" type="font/woff2" crossorigin;

            title {
                (title.map_or("maddie, wtf?!".into(), |title| format!("{} | maddie, wtf?!", title)))
            }
            style {
                (theme_header)
            }
        }
    }
}

pub async fn footer() -> Markup {
    let raw_hash = build_info::GIT_COMMIT_HASH.or(option_env!("COMMIT_HASH"));

    let (url, short_hash) = match raw_hash {
        Some(raw) if raw.ends_with("-dirty") && raw.len() >= 7 => {
            let url = format!(
                "https://github.com/maddiemort/maddie-wtf/tree/{}",
                &raw[0..7]
            );
            (url, Some(raw))
        }
        Some(raw) if raw.len() >= 7 => {
            let url = format!("https://github.com/maddiemort/maddie-wtf/tree/{}", raw);
            let short_hash = &raw[0..7];
            (url, Some(short_hash))
        }
        _ => ("https://github.com/maddiemort/maddie-wtf".to_owned(), None),
    };

    html! {
        footer class="sitefooter" {
            ul {
                @if let Some(hash) = short_hash {
                    li {
                        code { (hash) }
                    }
                }
                li {
                    code {
                        (env!("CARGO_PKG_NAME"))
                        " v"
                        (env!("CARGO_PKG_VERSION"))
                    }
                }
                li { a href=(url) { "source" } }
            }
        }
    }
}
