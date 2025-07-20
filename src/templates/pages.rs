use maud::{html, Markup};

use crate::{
    state::{
        render::{ChronoRef, PageRef, PostRef, PostsRef},
        Theme,
    },
    templates::wrappers,
};

pub async fn page(page: PageRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        page.metadata.title.as_deref(),
        theme,
        html! {
            (page)
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

pub async fn posts(posts: PostsRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some("posts"),
        theme,
        html! {
            (posts)
        },
    )
    .await
}

pub async fn chrono(chrono: ChronoRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some("chrono"),
        theme,
        html! {
            (chrono)
        },
    )
    .await
}

pub async fn not_found(theme: Theme) -> Markup {
    wrappers::base(
        Some("not found"),
        theme,
        html! {
            main class="error" {
                h1 {
                    "not found"
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
                h1 {
                    "internal server error"
                }

                p {
                    "wtf, you broke it?! stop doing that."
                }
            }
        },
    )
    .await
}
