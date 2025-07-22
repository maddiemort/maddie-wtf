use maud::{html, Markup, PreEscaped};

use crate::{
    state::{
        render::{
            ChronoRef, EntryRef, PageRef, PostRef, PostsRef, RecentPostsRef, RssFeedRef, TaggedRef,
            TagsRef,
        },
        Theme,
    },
    templates::wrappers,
};

pub async fn index(index: PageRef<'_>, recent_posts: RecentPostsRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        index.metadata.title.as_deref(),
        theme,
        html! {
            main {
                (index)
                (recent_posts)
            }
        },
    )
    .await
}

pub async fn page(page: PageRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        page.metadata.title.as_deref(),
        theme,
        html! {
            main {
                (page)
            }
        },
    )
    .await
}

pub async fn post(post: PostRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some(post.md_title()),
        theme,
        html! {
            (post)
        },
    )
    .await
}

pub async fn entry(entry: EntryRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some(entry.md_title()),
        theme,
        html! {
            main {
                (entry)
            }
        },
    )
    .await
}

pub async fn posts(posts: PostsRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some("Posts"),
        theme,
        html! {
            (posts)
        },
    )
    .await
}

pub async fn chrono(chrono: ChronoRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some("Chrono"),
        theme,
        html! {
            (chrono)
        },
    )
    .await
}

pub async fn tags(tags: TagsRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some("Tags"),
        theme,
        html! {
            (tags)
        },
    )
    .await
}

pub async fn tagged(tagged: TaggedRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some(&tagged.tag.to_string()),
        theme,
        html! {
            (tagged)
        },
    )
    .await
}

pub async fn rss_feed(rss_feed: RssFeedRef<'_>) -> Markup {
    // It's not HTML, it's XML, but we should be fine as long as we're careful.
    html! {
        (PreEscaped("<?xml version=\"1.0\" ?>"))
        rss version="2.0" {
            channel {
                title { "maddie, wtf?!" }
                link { "https://maddie.wtf" }
                description { "Madeleine Mortensen" }
                image {
                    url {
                        "https://maddie.wtf/static/favicon.svg"
                    }
                    link { "https://maddie.wtf" }
                    title { "Favicon" }
                }
                (rss_feed)
            }
        }
    }
}

pub async fn not_found(theme: Theme) -> Markup {
    wrappers::base(
        Some("not found"),
        theme,
        html! {
            main class="error" {
                h1 class="title" {
                    "Not Found"
                }

                p {
                    "wtf did you do?! that's not a route you can access."
                }
            }
        },
    )
    .await
}

pub async fn internal_error(theme: Theme) -> Markup {
    wrappers::base(
        Some("internal server error"),
        theme,
        html! {
            main class="error" {
                h1 class="title" {
                    "Internal Server Error"
                }

                p {
                    "wtf, you broke it?! stop doing that."
                }
            }
        },
    )
    .await
}
