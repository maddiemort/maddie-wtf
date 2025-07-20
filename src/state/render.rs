use std::{collections::HashMap, ops::Deref};

use camino::{Utf8Path, Utf8PathBuf};
use chrono::NaiveDate;
use maud::{html, Markup, PreEscaped, Render};
use tokio::sync::RwLockReadGuard;

use crate::state::{
    markdown_to_html, names::TagName, Node, Page, Post, SinglePostMetadata, ThreadEntryMetadata,
    ThreadMetadata,
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
                article {
                    (PreEscaped(post.html_title(1)))
                    ul class="frontmatter" {
                        li {
                            time datetime=(post.date_posted()) {
                                (post.date_posted().format("%e %B %Y"))
                            }
                        }
                        @for tag in post.tags() {
                            li {
                                // a href=(format!("/tagged/{}", tag)) {
                                    (tag)
                                // }
                            }
                        }
                    }
                    (PreEscaped(&html_content))
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
                    article {
                        (PreEscaped(post.html_title(1)))
                        @for (i, entry) in filtered_entries.iter().enumerate() {
                            @if i > 0 {
                                hr;
                            }
                            ul class="frontmatter" {
                                li {
                                    time datetime=(entry.metadata.date) {
                                        (entry.metadata.date.format("%e %B %Y"))
                                    }
                                }
                                @if i == 0 {
                                    @for tag in post.tags() {
                                        li {
                                            // a href=(format!("/tagged/{}", tag)) {
                                                (tag)
                                            // }
                                        }
                                    }
                                }
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

    pub fn into_chrono(self) -> ChronoRef<'a> {
        ChronoRef {
            guard: self.guard,
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
                hgroup {
                    h1 { "Posts" }
                    p {
                        (PreEscaped(&markdown_to_html(
                            "This is a list of posts in reverse chronological order by their \
                            original date of posting. If a post has been updated since then, its \
                            most recent update date is listed in its frontmatter, but if you want to \
                            see the updates broken out separately, you should visit \
                            [chrono](/chrono)."
                        )))
                    }
                }

                @for (path, post) in posts.iter().rev() {
                    hr;

                    section {
                        hgroup {
                            h2 {
                                ({
                                    let title = post.md_title();
                                    let md = format!("[{}](/posts/{})", title, path);
                                    PreEscaped(markdown_to_html(&md))
                                })
                            }
                            ul class="frontmatter" {
                                li {
                                    time datetime=(post.date_posted()) {
                                        (post.date_posted().format("%e %B %Y"))
                                    }
                                }
                                @if post.date_posted() != post.date_updated(self.show_drafts) {
                                    li {
                                        "updated "
                                        time datetime=(post.date_updated(self.show_drafts)) {
                                            (post.date_updated(self.show_drafts).format("%e %B %Y"))
                                        }
                                    }
                                }
                                @for tag in post.tags() {
                                    li {
                                        // a href=(format!("/tagged/{}", tag)) {
                                            (tag)
                                        // }
                                    }
                                }
                            }
                        }
                        (PreEscaped(post.summary()))
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
    fn date_updated(&self) -> NaiveDate {
        match self {
            ChronoEntry::Single { metadata, .. } => metadata.date,
            ChronoEntry::ThreadEntry { entry_meta, .. } => entry_meta.date,
        }
    }

    fn md_title(&self) -> &str {
        match self {
            ChronoEntry::Single { metadata, .. } => metadata.md_title.as_str(),
            ChronoEntry::ThreadEntry { thread_meta, .. } => thread_meta.md_title.as_str(),
        }
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
                hgroup {
                    h1 { "Chrono" }
                    p {
                        (PreEscaped(&markdown_to_html(
                            "This is a list of all updates made to posts in reverse chronological \
                            order, including the initial post and its additions and edits since. \
                            If you only want to see entire posts, you should visit [posts](/posts)."
                        )))
                    }
                }

                @for entry in entries.iter().rev() {
                    hr;

                    section {
                        hgroup {
                            h2 {
                                ({
                                    let title = entry.md_title();
                                    let md = format!("[{}](/posts/{})", title, entry.path());
                                    PreEscaped(markdown_to_html(&md))
                                })
                            }
                            ul class="frontmatter" {
                                li {
                                    time datetime=(entry.date_updated()) {
                                        (entry.date_updated().format("%e %B %Y"))
                                    }
                                }
                                @for tag in entry.tags() {
                                    li {
                                        // a href=(format!("/tagged/{}", tag)) {
                                            (tag)
                                        // }
                                    }
                                }
                            }
                        }
                        (PreEscaped(entry.summary()))
                    }
                }
            }
        }
    }
}
