use std::{collections::HashMap, ops::Deref};

use camino::{Utf8Path, Utf8PathBuf};
use chrono::NaiveDate;
use maud::{html, Markup, PreEscaped, Render};
use tokio::sync::RwLockReadGuard;

use crate::{
    state::{
        markdown_to_html, names::TagName, Node, Page, Post, SinglePostMetadata, ThreadEntry,
        ThreadEntryMetadata, ThreadMetadata,
    },
    templates::partials,
};

pub struct PostRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, Post>,
    pub(super) path: Utf8PathBuf,
    pub(super) show_drafts: bool,
}

impl<'a> PostRef<'a> {
    pub fn into_entry(self, index: usize, show_drafts: bool) -> Option<EntryRef<'a>> {
        if let Post::Thread { ref entries, .. } = *self {
            if index < entries.len() {
                // Ok, we know the entry exists. There are a few more checks to make, though.
                if !show_drafts && entries[index].metadata.draft {
                    // Simple: if the entry is a draft and we're not showing drafts, don't show it.
                    None
                } else if !show_drafts && entries.iter().filter(|e| !e.metadata.draft).count() == 1
                {
                    // If there's only one non-draft entry (regardless of whether there are more!),
                    // we shouldn't allow this entry to be displayed as an entry (because it will
                    // confuse readers, and it will leak that there might be more entries coming in
                    // the future). Just pretend it doesn't exist.
                    None
                } else {
                    Some(EntryRef {
                        guard: self.guard,
                        post_path: self.path,
                        index,
                    })
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Render for PostRef<'_> {
    fn render(&self) -> Markup {
        match self.guard.deref() {
            post @ Post::Single {
                metadata: _,
                html_summary: _,
                html_toc,
                html_content,
            } => html! {
                main {
                    article {
                        (partials::page_title(PreEscaped(post.html_title()), None))

                        (partials::post_frontmatter(
                            post.date_posted(),
                            post.date_updated(self.show_drafts),
                            post.tags(),
                        ))

                        hr;

                        @if let Some(toc) = html_toc {
                            (partials::table_of_contents(PreEscaped(toc.clone())))

                            hr;
                        }

                        (PreEscaped(&html_content))

                        @if post.lobsters().is_some()
                            || post.hacker_news().is_some() {
                            hr;
                        }

                        (partials::post_endmatter(post.lobsters(), post.hacker_news()))
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
                        @let multiple_entries = filtered_entries.len() > 1;
                        @let title_id = if multiple_entries {
                            Some("entry-0")
                        } else {
                            None
                        };

                        (partials::page_title(PreEscaped(post.html_title()), title_id))

                        (partials::post_frontmatter(
                            post.date_posted(),
                            post.date_updated(self.show_drafts),
                            post.tags(),
                        ))

                        @for (i, entry) in filtered_entries.iter().enumerate() {
                            @let has_next = i + 1 < filtered_entries.len();
                            @let has_prev = i > 0;

                            @if i > 0 {
                                hr;

                                @if let Some(entry_title) = entry.html_title() {
                                    (partials::page_title(
                                        PreEscaped(entry_title),
                                        Some(&format!("entry-{i}"))
                                    ))
                                }

                                @let index_for_id = if entry.metadata.md_title.is_none() {
                                    Some(i)
                                } else {
                                    None
                                };

                                (partials::post_entry_frontmatter(
                                    index_for_id,
                                    entry.metadata.date,
                                    entry.metadata.updated,
                                    post.tags(),
                                ))
                            }

                            @if multiple_entries {
                                (partials::entry_aside(i, &self.path, has_next, has_prev))
                            }

                            @if let Some(toc) = entry.html_toc.as_ref() {
                                hr;

                                (partials::table_of_contents(PreEscaped(toc.clone())))
                            }

                            hr;

                            (PreEscaped(&entry.html_content))

                            @if i == 0 {
                                @if post.lobsters().is_some() || post.hacker_news().is_some() {
                                    hr;
                                }

                                (partials::post_endmatter(
                                    post.lobsters(),
                                    post.hacker_news(),
                                ))
                            } @else {
                                @if entry.metadata.lobsters.is_some()
                                    || entry.metadata.hacker_news.is_some() {
                                    hr;
                                }

                                (partials::post_endmatter(
                                    entry.metadata.lobsters.as_ref(),
                                    entry.metadata.hacker_news.as_ref(),
                                ))
                            }
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

pub struct EntryRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, Post>,
    pub(super) post_path: Utf8PathBuf,
    pub(super) index: usize,
}

impl EntryRef<'_> {
    pub fn md_title(&self) -> &str {
        self.metadata
            .md_title
            .as_deref()
            .unwrap_or(self.thread_metadata().md_title.as_str())
    }

    pub fn html_title(&self) -> String {
        let html = markdown_to_html(self.md_title());

        html.strip_prefix("<p>")
            .and_then(|title| title.strip_suffix("</p>\n"))
            .map(|stripped| stripped.to_string())
            .unwrap_or(html)
    }

    pub fn thread_metadata(&self) -> &ThreadMetadata {
        let Post::Thread { metadata, .. } = self.guard.deref() else {
            unreachable!()
        };
        metadata
    }
}

impl Render for EntryRef<'_> {
    fn render(&self) -> Markup {
        html! {
            main {
                article {
                    (partials::page_title(PreEscaped(self.html_title()), None))

                    (partials::post_frontmatter(
                        self.metadata.date,
                        self.metadata.updated.unwrap_or(self.metadata.date),
                        self.thread_metadata().tags.iter(),
                    ))

                    aside {
                        em {
                            "You're reading a single entry in a longer post. The entire post is available "
                            a href=(format!("/posts/{}", self.post_path)) {
                                "here"
                            }
                            "."
                        }
                    }

                    hr;

                    @if let Some(ref toc) = self.html_toc {
                        (partials::table_of_contents(PreEscaped(toc.clone())))

                        hr;
                    }

                    (PreEscaped(&self.html_content))

                    hr;

                    (partials::post_endmatter(
                        self.metadata.lobsters.as_ref(),
                        self.metadata.hacker_news.as_ref(),
                    ))

                    aside {
                        em {
                            "You're reading a single entry in a longer post. The entire post is available "
                            a href=(format!("/posts/{}", self.post_path)) {
                                "here"
                            }
                            "."
                        }
                    }
                }
            }
        }
    }
}

impl Deref for EntryRef<'_> {
    type Target = ThreadEntry;

    fn deref(&self) -> &Self::Target {
        let Post::Thread { entries, .. } = self.guard.deref() else {
            unreachable!()
        };
        &entries[self.index]
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
                (partials::page_title(PreEscaped(title), None))
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

    pub fn into_recent_pubs(self) -> RecentPubsRef<'a> {
        RecentPubsRef {
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

    pub fn into_rss_feed(self) -> RssFeedRef<'a> {
        RssFeedRef {
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
                (partials::page_title(html! { "Posts" }, None))

                p {
                    "This is a list of posts in reverse chronological order by their original \
                    date of posting. If a post contains multiple entries, they'll all be shown on \
                    the linked page, but you can view each separately at "
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
                            post.tags(),
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

pub struct RecentPubsRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
    pub(super) show_drafts: bool,
}

impl Render for RecentPubsRef<'_> {
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
                        let mut entries_to_render = vec![];

                        let mut found_draft = false;
                        for (i, entry) in entries.iter().enumerate() {
                            found_draft |= entry.metadata.draft;

                            if self.show_drafts || !found_draft {
                                entries_to_render.push(ChronoEntry::ThreadEntry {
                                    post_path: path,
                                    index: i,
                                    display_as_entry: true,
                                    thread_meta: metadata,
                                    entry_meta: &entry.metadata,
                                    html_summary: entry.html_summary.as_str(),
                                });
                            }
                        }

                        if found_draft && entries_to_render.len() == 1 {
                            // Special case! There was more than one entry, but only the first one
                            // was not a draft. That means that if we display this first entry *as*
                            // an entry, we'll confuse readers (and tip them off that another entry
                            // might be coming). We shouldn't link to the entry page, we should
                            // just link to the main post.

                            let ChronoEntry::ThreadEntry {
                                ref mut display_as_entry,
                                ..
                            } = entries_to_render[0]
                            else {
                                unreachable!();
                            };

                            *display_as_entry = false;
                        }

                        to_render.extend(entries_to_render);
                    }
                    _ => {}
                }
                to_render
            })
            .collect::<Vec<ChronoEntry>>();
        entries.sort_by_key(|chrono_entry| chrono_entry.date_updated());

        html! {
            h1 { "Recent Publications" }

            ul {
                @for entry in entries.iter().rev().take(5) {
                    li {
                        a href=(entry.path()) {
                            (PreEscaped(entry.html_title()))
                        }
                        " (" (partials::date(entry.date_posted())) ")"
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
        post_path: &'a Utf8Path,
        index: usize,
        display_as_entry: bool,
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
            ChronoEntry::ThreadEntry {
                thread_meta,
                entry_meta,
                ..
            } => entry_meta
                .md_title
                .as_deref()
                .unwrap_or(thread_meta.md_title.as_str()),
        }
    }

    fn html_title(&self) -> String {
        let html = markdown_to_html(self.md_title());

        html.strip_prefix("<p>")
            .and_then(|title| title.strip_suffix("</p>\n"))
            .map(|stripped| stripped.to_string())
            .unwrap_or(html)
    }

    fn path(&self) -> String {
        match self {
            ChronoEntry::Single { path, .. } => {
                format!("/posts/{}", path)
            }
            ChronoEntry::ThreadEntry {
                post_path,
                index,
                display_as_entry,
                ..
            } => {
                if *display_as_entry {
                    format!("/posts/{}/entry/{}", post_path, index)
                } else {
                    format!("/posts/{}", post_path)
                }
            }
        }
    }

    fn rss_guid(&self) -> String {
        // RSS GUIDs are a little weird. We're going to pretend that any posts that are a single
        // entry are the first entry in a thread, because (1) the post might become a thread entry
        // in the future if another entry is published, (2) we want the GUID to remain stable even
        // if the post is converted into an entry, (3) RSS GUIDs don't have to be valid URLs
        // (especially if we send them with `isPermaLink="false"`), and (4) RSS GUIDs don't have to
        // be UUIDs, just strings that are "globally unique".
        match self {
            ChronoEntry::Single { path, .. } => format!("/posts/{}/entry/0", path),
            ChronoEntry::ThreadEntry {
                post_path, index, ..
            } => {
                format!("/posts/{}/entry/{}", post_path, index)
            }
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
                        let mut entries_to_render = vec![];

                        let mut found_draft = false;
                        for (i, entry) in entries.iter().enumerate() {
                            found_draft |= entry.metadata.draft;

                            if self.show_drafts || !found_draft {
                                entries_to_render.push(ChronoEntry::ThreadEntry {
                                    post_path: path,
                                    index: i,
                                    display_as_entry: true,
                                    thread_meta: metadata,
                                    entry_meta: &entry.metadata,
                                    html_summary: entry.html_summary.as_str(),
                                });
                            }
                        }

                        if found_draft && entries_to_render.len() == 1 {
                            // Special case! There was more than one entry, but only the first one
                            // was not a draft. That means that if we display this first entry *as*
                            // an entry, we'll confuse readers (and tip them off that another entry
                            // might be coming). We shouldn't link to the entry page, we should
                            // just link to the main post.

                            let ChronoEntry::ThreadEntry {
                                ref mut display_as_entry,
                                ..
                            } = entries_to_render[0]
                            else {
                                unreachable!();
                            };

                            *display_as_entry = false;
                        }

                        to_render.extend(entries_to_render);
                    }
                    _ => {}
                }
                to_render
            })
            .collect::<Vec<ChronoEntry>>();
        entries.sort_by_key(|chrono_entry| chrono_entry.date_updated());

        html! {
            main {
                (partials::page_title(html! { "Chrono" }, None))

                p {
                    "This is a list of all individual entries in posts in reverse chronological \
                    order, including the initial post and its additions since. If you only want \
                    to see entire posts, you should visit "
                    a href="/posts" { "posts" }
                    "."
                }

                @for entry in entries.iter().rev() {
                    hr;

                    section {
                        h2 {
                            a href=(entry.path()) {
                                (PreEscaped(entry.html_title()))
                            }
                        }
                        (partials::post_frontmatter(
                            entry.date_posted(),
                            entry.date_updated(),
                            entry.tags(),
                        ))
                        (PreEscaped(entry.summary()))
                        p {
                            a href=(entry.path()) {
                                "Read more"
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct RssFeedRef<'a> {
    pub(super) guard: RwLockReadGuard<'a, HashMap<Utf8PathBuf, Node>>,
    pub(super) show_drafts: bool,
}

impl Render for RssFeedRef<'_> {
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
                        let mut entries_to_render = vec![];

                        let mut found_draft = false;
                        for (i, entry) in entries.iter().enumerate() {
                            found_draft |= entry.metadata.draft;

                            if self.show_drafts || !found_draft {
                                entries_to_render.push(ChronoEntry::ThreadEntry {
                                    post_path: path,
                                    index: i,
                                    display_as_entry: true,
                                    thread_meta: metadata,
                                    entry_meta: &entry.metadata,
                                    html_summary: entry.html_summary.as_str(),
                                });
                            }
                        }

                        if found_draft && entries_to_render.len() == 1 {
                            // Special case! There was more than one entry, but only the first one
                            // was not a draft. That means that if we display this first entry *as*
                            // an entry, we'll confuse readers (and tip them off that another entry
                            // might be coming). We shouldn't link to the entry page, we should
                            // just link to the main post.

                            let ChronoEntry::ThreadEntry {
                                ref mut display_as_entry,
                                ..
                            } = entries_to_render[0]
                            else {
                                unreachable!();
                            };

                            *display_as_entry = false;
                        }

                        to_render.extend(entries_to_render);
                    }
                    _ => {}
                }
                to_render
            })
            .collect::<Vec<ChronoEntry>>();
        entries.sort_by_key(|chrono_entry| chrono_entry.date_updated());

        html! {
            @for entry in entries.iter().rev() {
                item {
                    title {
                        (PreEscaped(entry.md_title()))
                    }
                    pubDate {
                        (entry.date_posted().format("%a, %d %b %Y 00:00:00 +0000"))
                    }
                    link {
                        (format!("https://maddie.wtf{}", entry.path()))
                    }
                    guid isPermaLink="false" {
                        (entry.rss_guid())
                    }
                    description {
                        (entry.summary().replace('\n', " "))
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
                (partials::page_title(html! { "Tags" }, None))
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
                }, None))

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
                            post.tags(),
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
