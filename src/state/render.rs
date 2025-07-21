use std::{collections::HashMap, ops::Deref};

use camino::{Utf8Path, Utf8PathBuf};
use chrono::NaiveDate;
use maud::{html, Markup, PreEscaped, Render};
use tokio::sync::RwLockReadGuard;

use crate::{
    state::{
        markdown_to_html, names::TagName, Node, Page, Post, SinglePostMetadata,
        ThreadEntryMetadata, ThreadMetadata,
    },
    templates::partials,
};

pub struct PostRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, Post>,
    pub(super) show_drafts: bool,
}

impl Render for PostRef<'_> {
    fn render(&self) -> Markup {
        match self.guard.deref() {
            post @ Post::Single {
                metadata: _,
                html_summary: _,
                html_content,
            } => html! {
                main {
                    article {
                        (partials::page_title(PreEscaped(post.html_title())))
                        (partials::post_frontmatter(
                            post.date_posted(),
                            post.date_updated(self.show_drafts),
                            post.tags()
                        ))

                        (PreEscaped(&html_content))
                    }
                }
            },
            post @ Post::Thread {
                metadata: _,
                html_summary: _,
                entries,
            } => {
                let mut filtered_entries = vec![];
                let mut found_draft = false;

                for entry in entries {
                    found_draft |= entry.metadata.draft;
                    if self.show_drafts || !found_draft {
                        filtered_entries.push(entry);
                    }
                }

                html! {
                    main {
                        (partials::page_title(PreEscaped(post.html_title())))
                        (partials::post_frontmatter(
                            post.date_posted(),
                            post.date_updated(self.show_drafts),
                            post.tags()
                        ))

                        @for (i, entry) in filtered_entries.iter().enumerate() {
                            @if i > 0 {
                                hr;
                                (partials::post_entry_frontmatter(
                                    entry.metadata.date,
                                    entry.metadata.updated,
                                ))
                            }

                            (PreEscaped(&entry.html_content))
                        }
                    }
                }
            }
        }
    }
}

impl Deref for PostRef<'_> {
    type Target = Post;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct PageRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, Page>,
}

impl Render for PageRef<'_> {
    fn render(&self) -> Markup {
        let page = self.guard.deref();

        html! {
            @if let Some(title) = page.html_title() {
                (partials::page_title(PreEscaped(title)))
            }
            (PreEscaped(&page.html_content))
        }
    }
}

impl Deref for PageRef<'_> {
    type Target = Page;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

pub struct NodesRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
    pub(super) show_drafts: bool,
}

impl<'a> NodesRef<'a> {
    pub fn into_posts(self) -> PostsRef<'a> {
        PostsRef {
            guard: self.guard,
            show_drafts: self.show_drafts,
        }
    }

    pub fn into_recent_posts(self) -> RecentPostsRef<'a> {
        RecentPostsRef {
            guard: self.guard,
            show_drafts: self.show_drafts,
        }
    }

    pub fn into_chrono(self) -> ChronoRef<'a> {
        ChronoRef {
            guard: self.guard,
            show_drafts: self.show_drafts,
        }
    }

    pub fn into_tags(self) -> TagsRef<'a> {
        TagsRef {
            guard: self.guard,
            show_drafts: self.show_drafts,
        }
    }

    pub fn into_tagged(self, tag: TagName) -> TaggedRef<'a> {
        TaggedRef {
            guard: self.guard,
            tag,
            show_drafts: self.show_drafts,
        }
    }
}

pub struct PostsRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
    pub(super) show_drafts: bool,
}

impl Render for PostsRef<'_> {
    fn render(&self) -> Markup {
        let nodes = self.guard.deref();
        let mut posts = nodes
            .iter()
            .filter_map(|(path, node)| {
                if let Node::Post(post) = node {
                    Some((path.as_path(), post))
                } else {
                    None
                }
            })
            .filter(|(_, post)| {
                if !self.show_drafts {
                    // If we're not showing drafts, then filter out the following things:
                    //
                    // - Posts that are only a single entry that's a draft
                    // - Posts that are a thread where we can't display any of the entries (i.e. the
                    //   first entry is a draft, which implies the following are also drafts)
                    let is_draft = match post {
                        Post::Single { metadata, .. } => metadata.draft,
                        Post::Thread { entries, .. } => {
                            entries
                                .first()
                                .expect("a post cannot have no entries")
                                .metadata
                                .draft
                        }
                    };

                    !is_draft
                } else {
                    true
                }
            })
            .collect::<Vec<_>>();
        posts.sort_by_key(|(_, post)| post.date_posted());

        html! {
            main {
                (partials::page_title(html! { "Posts" }))

                p {
                    "This is a list of posts in reverse chronological order by their original \
                    date of posting. If a post has been updated since then, its most recent update \
                    date is listed in its frontmatter, but if you want to see the updates broken \
                    out separately, you should visit "
                    a href="/chrono" { "chrono" }
                    "."
                }

                @for (path, post) in posts.iter().rev() {
                    hr;

                    section {
                        h2 {
                            a href=(format!("/posts/{path}")) {
                                (PreEscaped(post.html_title()))
                            }
                        }
                        (partials::post_frontmatter(
                            post.date_posted(),
                            post.date_updated(self.show_drafts),
                            post.tags()
                        ))
                        (PreEscaped(post.summary()))
                        p {
                            a href=(format!("/posts/{}", path)) {
                                "Read more"
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct RecentPostsRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
    pub(super) show_drafts: bool,
}

impl Render for RecentPostsRef<'_> {
    fn render(&self) -> Markup {
        let nodes = self.guard.deref();
        let mut posts = nodes
            .iter()
            .filter_map(|(path, node)| {
                if let Node::Post(post) = node {
                    Some((path.as_path(), post))
                } else {
                    None
                }
            })
            .filter(|(_, post)| {
                if !self.show_drafts {
                    // If we're not showing drafts, then filter out the following things:
                    //
                    // - Posts that are only a single entry that's a draft
                    // - Posts that are a thread where we can't display any of the entries (i.e. the
                    //   first entry is a draft, which implies the following are also drafts)
                    let is_draft = match post {
                        Post::Single { metadata, .. } => metadata.draft,
                        Post::Thread { entries, .. } => {
                            entries
                                .first()
                                .expect("a post cannot have no entries")
                                .metadata
                                .draft
                        }
                    };

                    !is_draft
                } else {
                    true
                }
            })
            .collect::<Vec<_>>();
        posts.sort_by_key(|(_, post)| post.date_posted());

        html! {
            h1 { "Recent Posts" }

            ul {
                @for (path, post) in posts.iter().rev().take(5) {
                    li {
                        a href=(format!("/posts/{}", path)) {
                            (PreEscaped(post.html_title()))
                        }
                        " (" (partials::date(post.date_posted())) ")"
                    }
                }
            }
        }
    }
}

pub struct ChronoRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
    pub(super) show_drafts: bool,
}

enum ChronoEntry<'a> {
    Single {
        path: &'a Utf8Path,
        metadata: &'a SinglePostMetadata,
        html_summary: &'a str,
    },
    ThreadEntry {
        path: &'a Utf8Path,
        thread_meta: &'a ThreadMetadata,
        entry_meta: &'a ThreadEntryMetadata,
        html_summary: &'a str,
    },
}

impl ChronoEntry<'_> {
    fn date_posted(&self) -> NaiveDate {
        match self {
            ChronoEntry::Single { metadata, .. } => metadata.date,
            ChronoEntry::ThreadEntry { entry_meta, .. } => entry_meta.date,
        }
    }

    fn date_updated(&self) -> NaiveDate {
        match self {
            ChronoEntry::Single { metadata, .. } => metadata.updated.unwrap_or(metadata.date),
            ChronoEntry::ThreadEntry { entry_meta, .. } => {
                entry_meta.updated.unwrap_or(entry_meta.date)
            }
        }
    }

    fn md_title(&self) -> &str {
        match self {
            ChronoEntry::Single { metadata, .. } => metadata.md_title.as_str(),
            ChronoEntry::ThreadEntry { thread_meta, .. } => thread_meta.md_title.as_str(),
        }
    }

    fn html_title(&self) -> String {
        let html = markdown_to_html(self.md_title());

        html.strip_prefix("<p>")
            .and_then(|title| title.strip_suffix("</p>\n"))
            .map(|stripped| stripped.to_string())
            .unwrap_or(html)
    }

    fn path(&self) -> &Utf8Path {
        match self {
            ChronoEntry::Single { path, .. } | ChronoEntry::ThreadEntry { path, .. } => path,
        }
    }

    fn summary(&self) -> &str {
        match self {
            ChronoEntry::Single { html_summary, .. }
            | ChronoEntry::ThreadEntry { html_summary, .. } => html_summary,
        }
    }

    fn tags(&self) -> impl Iterator<Item = &TagName> {
        match self {
            ChronoEntry::Single { metadata, .. } => metadata.tags.iter(),
            ChronoEntry::ThreadEntry { thread_meta, .. } => thread_meta.tags.iter(),
        }
    }
}

impl Render for ChronoRef<'_> {
    fn render(&self) -> Markup {
        let nodes = self.guard.deref();
        let mut entries = nodes
            .iter()
            .flat_map(|(path, node)| {
                let mut to_render = vec![];
                match node {
                    Node::Post(Post::Single {
                        metadata,
                        html_summary,
                        ..
                    }) => {
                        if self.show_drafts || !metadata.draft {
                            to_render.push(ChronoEntry::Single {
                                path,
                                metadata,
                                html_summary: html_summary.as_str(),
                            });
                        }
                    }
                    Node::Post(Post::Thread {
                        metadata, entries, ..
                    }) => {
                        let mut found_draft = false;
                        for entry in entries {
                            found_draft |= entry.metadata.draft;

                            if self.show_drafts || !found_draft {
                                to_render.push(ChronoEntry::ThreadEntry {
                                    path,
                                    thread_meta: metadata,
                                    entry_meta: &entry.metadata,
                                    html_summary: entry.html_summary.as_str(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
                to_render
            })
            .collect::<Vec<ChronoEntry>>();
        entries.sort_by_key(|chrono_entry| chrono_entry.date_updated());

        html! {
            main {
                (partials::page_title(html! { "Chrono" }))

                p {
                    "This is a list of all updates made to posts in reverse chronological \
                    order, including the initial post and its additions and edits since. If \
                    you only want to see entire posts, you should visit "
                    a href="/posts" { "posts" }
                    "."
                }

                @for entry in entries.iter().rev() {
                    hr;

                    section {
                        h2 {
                            a href=(format!("/posts/{}", entry.path())) {
                                (PreEscaped(entry.html_title()))
                            }
                        }
                        (partials::post_frontmatter(
                            entry.date_posted(),
                            entry.date_updated(),
                            entry.tags()
                        ))
                        (PreEscaped(entry.summary()))
                        p {
                            a href=(format!("/posts/{}", entry.path())) {
                                "Read more"
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct TagsRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
    pub(super) show_drafts: bool,
}

impl Render for TagsRef<'_> {
    fn render(&self) -> Markup {
        let nodes = self.guard.deref();

        let mut tags = HashMap::<TagName, Vec<_>>::new();

        for (path, post) in nodes
            .iter()
            .filter_map(|(path, node)| {
                if let Node::Post(post) = node {
                    Some((path.as_path(), post))
                } else {
                    None
                }
            })
            .filter(|(_, post)| {
                if !self.show_drafts {
                    // If we're not showing drafts, then filter out the following things:
                    //
                    // - Posts that are only a single entry that's a draft
                    // - Posts that are a thread where we can't display any of the entries (i.e. the
                    //   first entry is a draft, which implies the following are also drafts)
                    let is_draft = match post {
                        Post::Single { metadata, .. } => metadata.draft,
                        Post::Thread { entries, .. } => {
                            entries
                                .first()
                                .expect("a post cannot have no entries")
                                .metadata
                                .draft
                        }
                    };

                    !is_draft
                } else {
                    true
                }
            })
        {
            for tag in post.tags() {
                tags.entry(tag.clone()).or_default().push((path, post));
            }
        }

        let mut tags_list = tags.iter().collect::<Vec<_>>();
        tags_list.sort_by_key(|(name, _)| *name);

        html! {
            main {
                (partials::page_title(html! { "Tags" }))
                p {
                    "This is a list of all tags found on "
                    a href="/posts" { "posts" }
                    "."
                }

                hr;

                ul {
                    @for (tag, posts) in tags_list {
                        @let posts_len = posts.len();
                        li {
                            a href=(format!("/tagged/{}", tag)) {
                                code { (tag) }
                            }
                            " ("
                            (posts_len)
                            @if posts_len == 1 {
                                " post"
                            } @else {
                                " posts"
                            }
                            ")"
                        }
                    }
                }
            }
        }
    }
}

pub struct TaggedRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
    pub(crate) tag: TagName,
    pub(super) show_drafts: bool,
}

impl Render for TaggedRef<'_> {
    fn render(&self) -> Markup {
        let nodes = self.guard.deref();
        let mut posts = nodes
            .iter()
            .filter_map(|(path, node)| {
                if let Node::Post(post) = node {
                    Some((path.as_path(), post))
                } else {
                    None
                }
            })
            .filter(|(_, post)| {
                if !self.show_drafts {
                    // If we're not showing drafts, then filter out the following things:
                    //
                    // - Posts that are only a single entry that's a draft
                    // - Posts that are a thread where we can't display any of the entries (i.e. the
                    //   first entry is a draft, which implies the following are also drafts)
                    let is_draft = match post {
                        Post::Single { metadata, .. } => metadata.draft,
                        Post::Thread { entries, .. } => {
                            entries
                                .first()
                                .expect("a post cannot have no entries")
                                .metadata
                                .draft
                        }
                    };

                    !is_draft
                } else {
                    true
                }
            })
            .filter(|(_, post)| post.has_tag(&self.tag))
            .collect::<Vec<_>>();
        posts.sort_by_key(|(_, post)| post.date_posted());

        html! {
            main {
                (partials::page_title(html! {
                    "Posts Tagged " code { (self.tag) }
                }))

                p {
                    "This is a list of all posts tagged with "
                    code { (self.tag) }
                    ", in reverse chronological order by their original date of posting. If a \
                    post has been updated since then, its most recent update date is listed \
                    in its frontmatter."
                }

                @for (path, post) in posts.iter().rev() {
                    hr;

                    section {
                        h2 {
                            a href=(format!("/posts/{path}")) {
                                (PreEscaped(post.html_title()))
                            }
                        }
                        (partials::post_frontmatter(
                            post.date_posted(),
                            post.date_updated(self.show_drafts),
                            post.tags()
                        ))
                        (PreEscaped(post.summary()))
                        p {
                            a href=(format!("/posts/{}", path)) {
                                "Read more"
                            }
                        }
                    }
                }
            }
        }
    }
}
