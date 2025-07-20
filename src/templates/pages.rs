use maud::{html, Markup};

use crate::{
    state::{
        render::{ChronoRef, PageRef, PostRef, PostsRef, RecentPostsRef, TagsRef},
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

pub async fn tags(tags: TagsRef<'_>, theme: Theme) -> Markup {
    wrappers::base(
        Some("tags"),
        theme,
        html! {
            (tags)
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
                h1 {
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
