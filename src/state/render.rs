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
}

impl<'a> Render for PostRef<'a> {
    fn render(&self) -> Markup {
        match self.guard.deref() {
            post @ Post::Single {
                metadata,
                html_summary: _,
                html_content,
            } => html! {
                article {
                    (PreEscaped(post.html_title(1)))
                    ul class="frontmatter" {
                        li { (metadata.date) }
                        @for tag in post.tags() {
                            li {
                                a href=(format!("/tagged/{}", tag)) {
                                    (tag)
                                }
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
            } => html! {
                article {
                    (PreEscaped(post.html_title(1)))
                    @for (i, entry) in entries.iter().enumerate() {
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
                                        a href=(format!("/tagged/{}", tag)) {
                                            (tag)
                                        }
                                    }
                                }
                            }
                        }
                        p {
                            (PreEscaped(&entry.html_content))
                        }
                    }
                }
            },
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

impl<'a> Render for PageRef<'a> {
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
}

impl<'a> NodesRef<'a> {
    pub fn into_posts(self) -> PostsRef<'a> {
        PostsRef { guard: self.guard }
    }

    pub fn into_chrono(self) -> ChronoRef<'a> {
        ChronoRef { guard: self.guard }
    }
}

pub struct PostsRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
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
            .collect::<Vec<_>>();
        posts.sort_by_key(|(_, post)| post.date_posted());

        html! {
            main {
                hgroup {
                    h1 { "posts" }
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
                                @if post.date_posted() != post.date_updated() {
                                    li {
                                        "updated "
                                        time datetime=(post.date_updated()) {
                                            (post.date_updated().format("%e %B %Y"))
                                        }
                                    }
                                }
                                @for tag in post.tags() {
                                    li {
                                        a href=(format!("/tagged/{}", tag)) {
                                            (tag)
                                        }
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
            .flat_map(|(path, node)| match node {
                Node::Post(Post::Single {
                    metadata,
                    html_summary,
                    ..
                }) => vec![ChronoEntry::Single {
                    path,
                    metadata,
                    html_summary: html_summary.as_str(),
                }],
                Node::Post(Post::Thread {
                    metadata, entries, ..
                }) => entries
                    .iter()
                    .map(|entry| ChronoEntry::ThreadEntry {
                        path,
                        thread_meta: metadata,
                        entry_meta: &entry.metadata,
                        html_summary: entry.html_summary.as_str(),
                    })
                    .collect(),
                _ => vec![],
            })
            .collect::<Vec<ChronoEntry>>();
        entries.sort_by_key(|chrono_entry| chrono_entry.date_updated());

        html! {
            main {
                hgroup {
                    h1 { "chrono" }
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
                                        a href=(format!("/tagged/{}", tag)) {
                                            (tag)
                                        }
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
