use std::{env, option_env};

use camino::Utf8Path;
use chrono::NaiveDate;
use maddie_wtf::build_info;
use maud::{html, Markup};

use crate::state::{names::TagName, Theme};

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

            link rel="alternate" type="application/rss+xml" href="/rss.xml" title="maddie, wtf?!";

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
                li { a href="/rss.xml" { "feed" } }
                li {
                    code {
                        (env!("CARGO_PKG_NAME"))
                    }
                    " "
                    code {
                        "v"
                        (env!("CARGO_PKG_VERSION"))
                    }
                    @if let Some(hash) = short_hash {
                        " "
                        code { (hash) }
                    }
                }
                li { a href=(url) { "source" } }
            }
        }
    }
}

pub fn page_title(html_title: Markup, title_id: Option<&str>) -> Markup {
    html! {
        @if let Some(id) = title_id {
            h1 class="title" id=(id) {
                (html_title)
            }
        } @else {
            h1 class="title" {
                (html_title)
            }
        }
    }
}

pub fn post_frontmatter<'a>(
    date_posted: NaiveDate,
    date_updated: NaiveDate,
    tags: impl Iterator<Item = &'a TagName>,
) -> Markup {
    html! {
        ul class="frontmatter" {
            li {
                (self::date_posted(date_posted))
            }

            @if date_posted != date_updated {
                li {
                    (self::date_updated(date_updated))
                }
            }

            (tag_list(tags))
        }
    }
}

pub fn post_entry_frontmatter<'a>(
    index: Option<usize>,
    date_posted: NaiveDate,
    date_updated: Option<NaiveDate>,
    tags: impl Iterator<Item = &'a TagName>,
) -> Markup {
    fn ul_optional_id(index: Option<usize>, body: Markup) -> Markup {
        html! {
            @if let Some(index) = index {
                ul id=(format!("entry-{index}")) class="frontmatter" {
                    (body)
                }
            } @else {
                ul class="frontmatter" {
                    (body)
                }
            }
        }
    }

    ul_optional_id(
        index,
        html! {
            li {
                (self::date_posted(date_posted))
            }

            @if let Some(updated) = date_updated {
                @if date_posted != updated {
                    li {
                        (self::date_updated(updated))
                    }
                }
            }

            (tag_list(tags))
        },
    )
}

fn date_posted(date: NaiveDate) -> Markup {
    html! {
        em {
            "Posted " (self::date(date))
        }
    }
}

fn date_updated(date: NaiveDate) -> Markup {
    html! {
        em {
            "Updated " (self::date(date))
        }
    }
}

pub fn date(date: NaiveDate) -> Markup {
    html! {
        time datetime=(date) {
            (date.format("%d %B %Y"))
        }
    }
}

fn tag_list<'a>(tags: impl Iterator<Item = &'a TagName>) -> Markup {
    html! {
        @for tag in tags {
            li {
                a href=(format!("/tagged/{}", tag)) {
                    code { (tag) }
                }
            }
        }
    }
}

pub fn table_of_contents(toc_items: Markup) -> Markup {
    html! {
        nav class="toc" {
            h2 { "Table of Contents" }
            ul id="toc-list" {
                (toc_items)
            }
        }
    }
}

pub fn entry_aside(index: usize, path: &Utf8Path, has_next: bool, has_prev: bool) -> Markup {
    html! {
        aside {
            em {
                "This post contains multiple entries. You can "
                a href=(format!("/posts/{}/entry/{}", path, index)) {
                    "view this entry on its own"
                }

                @if has_prev {
                    @if has_next {
                        ","
                    } @else {
                        " or"
                    }

                    " jump to the "
                    a href=(format!("#entry-{}", index - 1)) {
                        "previous entry"
                    }
                }

                @if has_next {
                    @if has_prev {
                        ", or"
                    } @else {
                        " or"
                    }

                    " jump to the "
                    a href=(format!("#entry-{}", index + 1)) {
                        "next entry"
                    }
                }

                "."
            }
        }
    }
}
