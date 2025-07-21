use std::{
    collections::HashMap, fs::Metadata, io, path::StripPrefixError, sync::Arc, time::Duration,
};

use axum::extract::FromRef;
use camino::{Utf8Path, Utf8PathBuf};
use chrono::naive::NaiveDate;
use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter, ComrakOptions, ComrakPlugins,
};
use either::Either;
use ignore::Walk;
use lazy_static::lazy_static;
use maud::{html, Markup, PreEscaped};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer};
use serde::Deserialize;
use syntect::{
    highlighting::ThemeSet as SyntectThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle},
    Error as SyntectError, LoadingError as SyntectLoadingError,
};
use thiserror::Error;
use tokio::{
    fs, runtime,
    sync::{RwLock, RwLockReadGuard},
    task::JoinHandle,
};
use tower_livereload::Reloader;
use tracing::{debug, error, info, instrument, span, warn, Level};

use crate::{
    state::{
        names::TagName,
        render::{NodesRef, PageRef, PostRef},
    },
    Args,
};

pub mod names;
pub mod render;

lazy_static! {
    static ref SYNTECT_ADAPTER: SyntectAdapter = SyntectAdapter::new(None);
    static ref COMRAK_PLUGINS: ComrakPlugins<'static> = {
        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&*SYNTECT_ADAPTER);
        plugins
    };
    static ref COMRAK_OPTIONS: ComrakOptions = ComrakOptions::default();
}

fn markdown_to_html(md_input: &str) -> String {
    markdown_to_html_with_plugins(md_input, &COMRAK_OPTIONS, &COMRAK_PLUGINS)
}

#[derive(Clone, Debug)]
pub struct Config {
    pub drafts: bool,
    pub content_path: Utf8PathBuf,
    pub static_path: Utf8PathBuf,
    pub themes_path: Utf8PathBuf,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        let Args {
            drafts,
            content_path,
            static_path,
            themes_path,
            ..
        } = args;
        Self {
            drafts,
            content_path: content_path
                .canonicalize_utf8()
                .expect("should be able to canonicalize content path"),
            static_path: static_path
                .canonicalize_utf8()
                .expect("should be able to canonicalize static path"),
            themes_path: themes_path
                .canonicalize_utf8()
                .expect("should be able to canonicalize themes path"),
        }
    }
}

impl Config {
    pub async fn load_state(self, reloader: Reloader) -> Result<State, LoadStateError> {
        use LoadStateError::*;

        let theme_set = SyntectThemeSet::load_from_folder(self.themes_path)?;
        let theme = Theme::try_load(theme_set, "OneHalfLight", "OneHalfDark")?;

        let content = Content::empty_in(self.content_path.clone());

        let walker = Walk::new(&self.content_path);
        for result in walker {
            match result {
                Ok(entry) => {
                    let Ok(path) = Utf8PathBuf::from_path_buf(entry.path().to_path_buf()) else {
                        warn!(
                            path = ?entry.path(),
                            "skipping entry with path that contains invalid UTF-8"
                        );
                        continue;
                    };

                    let Ok(metadata) = entry.metadata() else {
                        warn!(%path, "skipping entry without valid metadata");
                        continue;
                    };

                    if let Err(error) = content.load(path, metadata).await {
                        warn!(%error, "failed to load content");
                    }
                }
                Err(error) => error!(%error, "directory walker encountered error"),
            }
        }

        let (event_tx, event_rx) = std::sync::mpsc::channel::<DebouncedEvent>();

        let runtime = runtime::Handle::current();
        let content_1 = content.clone();
        let content_path_1 = self.content_path.clone();

        let loader_handle = runtime.spawn_blocking(move || {
            let _guard = span!(Level::ERROR, "content_loader").entered();
            let runtime = runtime::Handle::current();
            while let Ok(event) = event_rx.recv() {
                runtime.block_on(async {
                    let Ok(path) = Utf8PathBuf::from_path_buf(event.path.clone()) else {
                        warn!(
                            path = ?event.path,
                            "skipping event with path that contains invalid UTF-8"
                        );
                        return;
                    };

                    let Ok(relative) = path.strip_prefix(&content_path_1) else {
                        debug!(
                            %path,
                            "skipping entry for path that isn't relative to the content path"
                        );
                        return;
                    };

                    if relative
                        .components()
                        .any(|component| component.as_str().starts_with('.'))
                    {
                        debug!(
                            %path,
                            "skipping entry for a path containing a hidden file or directory"
                        );
                        return;
                    }

                    if path
                        .file_name()
                        .is_some_and(|name| name == "4913" || name.ends_with('~'))
                    {
                        // nvim creates these when you write files. I think the ~ one is
                        // intentional, but the 4913 thing seems to be a longstanding bug:
                        //
                        // https://github.com/neovim/neovim/issues/3460
                        debug!(
                            %path,
                            "skipping entry that appears to be an editor temporary file"
                        );
                        return;
                    }

                    if !fs::try_exists(&path).await.unwrap_or_default() {
                        warn!(%path, "event probably represents a deleted file");
                        // TODO: handle deletions
                    } else {
                        let Ok(metadata) = fs::metadata(&path).await else {
                            warn!(
                                %path,
                                "skipping entry because metadata could not be accessed"
                            );
                            return;
                        };

                        match content_1.load(path, metadata).await {
                            Ok(_) => {
                                #[cfg(debug_assertions)]
                                {
                                    info!("sending reload");
                                    reloader.reload();
                                }
                            }
                            Err(error) => {
                                warn!(%error, "failed to load content");
                            }
                        }
                    }
                });
            }

            warn!("event sender hung up");
        });

        let mut watcher = new_debouncer(
            Duration::from_millis(25),
            move |res: DebounceEventResult| {
                let _guard = span!(Level::ERROR, "file_watcher").entered();
                match res {
                    Ok(events) => {
                        info!(events = %events.len(), "received batch of debounced events");
                        for event in events {
                            if let Err(error) = event_tx.send(event) {
                                error!(%error, "failed to send event to content loader");
                            }
                        }
                    }
                    Err(error) => error!(%error, "watcher error received"),
                }
            },
        )
        .map_err(CreateWatcher)?;

        watcher
            .watcher()
            .watch(self.content_path.as_std_path(), RecursiveMode::Recursive)
            .map_err(WatchPath)?;

        let settings = Settings {
            show_drafts: self.drafts,
        };

        Ok(State {
            content,
            theme,
            settings,
            _watcher: Arc::new(watcher),
            _loader_handle: Arc::new(loader_handle),
        })
    }
}

#[derive(Error, Debug)]
pub enum LoadStateError {
    #[error("failed to load theme set: {0}")]
    LoadThemeSet(#[from] SyntectLoadingError),

    #[error(transparent)]
    LoadThemeError(#[from] LoadThemeError),

    #[error("failed to create notify watcher: {0}")]
    CreateWatcher(#[source] notify::Error),

    #[error("failed to watch new path: {0}")]
    WatchPath(#[source] notify::Error),
}

#[derive(Clone, Debug)]
pub struct State {
    pub content: Content,
    pub theme: Theme,
    pub settings: Settings,
    _watcher: Arc<Debouncer<RecommendedWatcher>>,
    _loader_handle: Arc<JoinHandle<()>>,
}

#[derive(Clone, Debug)]
pub struct Content {
    root: Arc<Utf8PathBuf>,
    nodes: Arc<RwLock<HashMap<Utf8PathBuf, Node>>>,
}

impl Content {
    /// Create a new empty set of content, but with the root path set to `root`.
    pub fn empty_in(root: Utf8PathBuf) -> Self {
        Self {
            root: Arc::new(root),
            nodes: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    #[instrument(name = "load_content", level = "ERROR", skip_all)]
    pub async fn load<P>(&self, path: P, metadata: Metadata) -> Result<(), LoadContentError>
    where
        P: AsRef<Utf8Path>,
    {
        let path = path.as_ref();

        let mut nodes_guard = self.nodes.write().await;

        // All the nodes will be keyed by their paths relative to the content root, without an
        // extension.
        //
        // For now, keep the extension, so we'll be able to reconstruct the actual on-disk path by
        // joining the two together later.
        let relative_path = path
            .strip_prefix(&*self.root)
            .map_err(LoadContentError::NotRelative)?
            .to_owned();

        if metadata.is_file() {
            let file_name = relative_path
                .file_stem()
                .ok_or(LoadContentError::NoFileName)?;
            let file_ext = relative_path
                .extension()
                .ok_or(LoadContentError::NoExtension)?;

            if file_ext == "md" {
                if let Ok((date, _)) = NaiveDate::parse_and_remainder(file_name, "%Y-%m-%d") {
                    debug!(%relative_path, "loading post from file");
                    match self.load_post(&relative_path, date).await {
                        Ok(post) => {
                            nodes_guard.insert(relative_path.with_extension(""), Node::Post(post));
                            Ok(())
                        }
                        Err(error) => Err(error.into()),
                    }
                } else {
                    debug!(%relative_path, "loading page from file");
                    match self.load_page(&relative_path).await {
                        Ok(page) => {
                            nodes_guard.insert(relative_path.with_extension(""), Node::Page(page));
                            Ok(())
                        }
                        Err(error) => Err(error.into()),
                    }
                }
            } else {
                info!(%relative_path, "skipping non-markdown file");
                Ok(())
            }
        } else if metadata.is_dir() {
            warn!(%relative_path, "skipping directory");
            Ok(())
        } else {
            warn!(%relative_path, "skipping entry that is neither a file nor directory");
            Ok(())
        }
    }

    async fn load_post(
        &self,
        relative_path: &Utf8Path,
        date: NaiveDate,
    ) -> Result<Post, LoadPostError> {
        use LoadPostError::*;

        let raw_content = fs::read_to_string(self.root.join(relative_path))
            .await
            .map_err(ReadContent)?;

        let (first_raw_fm, mut rest) = raw_content
            .strip_prefix("---")
            .ok_or(MissingFrontmatter)?
            .split_once("---")
            .ok_or(MalformedFrontmatter)?;

        let first_frontmatter = toml::from_str::<PostFrontmatter>(first_raw_fm.trim())?;
        let mut metadata: Either<
            SinglePostMetadata,
            (ThreadMetadata, Vec<ThreadEntryMetadata>, Vec<&str>),
        > = Either::Left(SinglePostMetadata {
            md_title: first_frontmatter.md_title,
            draft: first_frontmatter.draft,
            tags: first_frontmatter.tags,
            date,
        });

        while let Some((last_content, (this_raw_frontmatter, new_rest))) = rest
            .split_once("---")
            .and_then(|(last_content, fm_and_rest)| {
                fm_and_rest
                    .split_once("---")
                    .map(|split_fm_rest| (last_content, split_fm_rest))
            })
        {
            rest = new_rest;

            let this_metadata = toml::from_str::<ThreadEntryMetadata>(this_raw_frontmatter.trim())?;

            match metadata {
                Either::Left(single) => {
                    let (thread_meta, first_meta) = single.split_for_thread();
                    metadata = Either::Right((
                        thread_meta,
                        vec![first_meta, this_metadata],
                        vec![last_content.trim()],
                    ));
                }
                Either::Right((_, ref mut entries, ref mut content)) => {
                    entries.push(this_metadata);
                    content.push(last_content);
                }
            }
        }

        match metadata {
            Either::Left(metadata) => {
                let rest = rest.trim();

                let mut raw_summary_paras = Vec::new();
                for (i, par) in rest.split("\n\n").enumerate() {
                    if par.starts_with('#') && i == 0 {
                        // This is a heading, but it's the first one, so just skip it
                        continue;
                    } else if par.starts_with('#') || par == "<!-- cut -->" {
                        // We've hit the next heading or a manual summary cut, so the summary
                        // should stop
                        break;
                    } else {
                        raw_summary_paras.push(par);
                    }

                    if raw_summary_paras.len() == 2 {
                        break;
                    }
                }

                let raw_summary = raw_summary_paras.join("\n\n");
                let html_summary = markdown_to_html(&raw_summary);

                let html_content = markdown_to_html(rest);

                let post = Post::Single {
                    metadata,
                    html_summary,
                    html_content,
                };

                info!(%relative_path, "loaded single post");
                Ok(post)
            }
            Either::Right((thread_meta, entry_metas, mut entry_raw_content)) => {
                entry_raw_content.push(rest.trim());

                let mut raw_summary_paras = Vec::new();
                for (i, par) in entry_raw_content
                    .first()
                    .expect("threaded post has at least one entry")
                    .split("\n\n")
                    .enumerate()
                {
                    if par.starts_with('#') && i == 0 {
                        // This is a heading, but it's the first one, so just skip it
                        continue;
                    } else if par.starts_with('#') || par == "<!-- cut -->" {
                        // We've hit the next heading or a manual summary cut, so the summary
                        // should stop
                        break;
                    } else {
                        raw_summary_paras.push(par);
                    }

                    if raw_summary_paras.len() == 2 {
                        break;
                    }
                }

                let raw_summary = raw_summary_paras.join("\n\n");
                let html_summary = markdown_to_html(&raw_summary);

                let entries = entry_metas
                    .into_iter()
                    .zip(entry_raw_content.into_iter())
                    .map(|(metadata, raw_content)| {
                        let raw_content = raw_content.trim();
                        let raw_summary = raw_content
                            .split("\n\n")
                            .filter(|par| !par.starts_with('#'))
                            .take(2)
                            .collect::<Vec<_>>()
                            .join("\n\n");
                        let html_summary = markdown_to_html(&raw_summary);

                        let html_content = markdown_to_html(raw_content);

                        ThreadEntry {
                            metadata,
                            html_summary,
                            html_content,
                        }
                    })
                    .collect::<Vec<_>>();
                let entries_len = entries.len();

                let post = Post::Thread {
                    metadata: thread_meta,
                    html_summary,
                    entries,
                };

                info!(entries = %entries_len, %relative_path, "loaded threaded post");
                Ok(post)
            }
        }
    }

    async fn load_page(&self, relative_path: &Utf8Path) -> Result<Page, LoadPageError> {
        use LoadPageError::*;

        let raw_content = fs::read_to_string(self.root.join(relative_path))
            .await
            .map_err(ReadContent)?;

        let (frontmatter, raw_content) = raw_content
            .strip_prefix("---")
            .ok_or(MissingFrontmatter)?
            .split_once("---")
            .ok_or(MalformedFrontmatter)?;

        let metadata = toml::from_str::<PageMetadata>(frontmatter.trim())?;
        let html_content = markdown_to_html(raw_content);

        let page = Page {
            metadata,
            html_content,
        };

        info!(%relative_path, "loaded page");
        Ok(page)
    }

    pub async fn post<P>(&self, path: P, show_drafts: bool) -> Option<PostRef<'_>>
    where
        P: AsRef<Utf8Path>,
    {
        let nodes_guard = self.nodes.read().await;
        let post_guard = RwLockReadGuard::try_map(nodes_guard, |nodes| {
            nodes.get(path.as_ref()).and_then(|node| {
                if let Node::Post(post) = node {
                    if show_drafts || !post.is_entirely_draft() {
                        Some(post)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        });

        if let Ok(post_guard) = post_guard {
            Some(PostRef {
                guard: post_guard,
                show_drafts,
            })
        } else {
            None
        }
    }

    pub async fn page<P>(&self, path: P) -> Option<PageRef<'_>>
    where
        P: AsRef<Utf8Path>,
    {
        let nodes_guard = self.nodes.read().await;
        let page_guard = RwLockReadGuard::try_map(nodes_guard, |nodes| {
            nodes.get(path.as_ref()).and_then(|node| {
                if let Node::Page(page) = node {
                    Some(page)
                } else {
                    None
                }
            })
        });

        if let Ok(page_guard) = page_guard {
            Some(PageRef { guard: page_guard })
        } else {
            None
        }
    }

    pub async fn nodes(&self, show_drafts: bool) -> NodesRef<'_> {
        let nodes_guard = self.nodes.read().await;
        NodesRef {
            guard: nodes_guard,
            show_drafts,
        }
    }

    pub async fn tag_exists(&self, tag: &TagName) -> bool {
        self.nodes.read().await.iter().any(|(_, node)| {
            if let Node::Post(post) = node {
                post.has_tag(tag)
            } else {
                false
            }
        })
    }
}

impl FromRef<State> for Content {
    fn from_ref(input: &State) -> Self {
        input.content.clone()
    }
}

#[derive(Debug, Error)]
pub enum LoadContentError {
    #[error("path doesn't contain a file name")]
    NoFileName,

    #[error("path to file doesn't appear to be relative to the content path")]
    NotRelative(#[source] StripPrefixError),

    #[error("path doesn't contain a file extension")]
    NoExtension,

    #[error(transparent)]
    LoadPost(#[from] LoadPostError),

    #[error(transparent)]
    LoadPage(#[from] LoadPageError),
}

#[derive(Clone, Debug)]
pub enum Node {
    Post(Post),
    Page(Page),
}

#[derive(Clone, Debug)]
pub enum Post {
    Single {
        metadata: SinglePostMetadata,
        html_summary: String,
        html_content: String,
    },
    Thread {
        metadata: ThreadMetadata,
        html_summary: String,
        entries: Vec<ThreadEntry>,
    },
}

impl Post {
    pub fn md_title(&self) -> &str {
        match self {
            Post::Single { metadata, .. } => &metadata.md_title,
            Post::Thread { metadata, .. } => &metadata.md_title,
        }
    }

    pub fn html_title(&self, levels: Option<usize>) -> String {
        if let Some(levels) = levels {
            markdown_to_html(&format!("{} {}", "#".repeat(levels), self.md_title()))
        } else {
            let html = markdown_to_html(self.md_title());

            html.strip_prefix("<p>")
                .and_then(|title| title.strip_suffix("</p>\n"))
                .map(|stripped| stripped.to_string())
                .unwrap_or(html)
        }
    }

    pub fn summary(&self) -> &str {
        match self {
            Post::Single { html_summary, .. } | Post::Thread { html_summary, .. } => html_summary,
        }
    }

    pub fn date_posted(&self) -> NaiveDate {
        match self {
            Post::Single { metadata, .. } => metadata.date,
            Post::Thread { entries, .. } => {
                entries
                    .first()
                    .expect("a post cannot have no entries")
                    .metadata
                    .date
            }
        }
    }

    pub fn date_updated(&self, include_draft_entries: bool) -> NaiveDate {
        match self {
            Post::Single { metadata, .. } => metadata.date,
            Post::Thread { entries, .. } => {
                // If we're not including draft entries, then return the date of the last entry
                // before the first one marked as a draft.
                //
                // If all entries are drafts (or the first entry is a draft, which is treated as
                // equivalent), return the date of the first entry because hopefully the caller
                // won't be displaying this at all if they know they're not supposed to display
                // drafts.
                let entry_for_date = if include_draft_entries {
                    entries.first().expect("a post cannot have no entries")
                } else {
                    entries
                        .iter()
                        .fold(
                            (
                                false,
                                entries.first().expect("a post cannot have no entries"),
                            ),
                            |(found_draft, acc), next| {
                                let found_draft = found_draft || next.metadata.draft;

                                // If we still haven't found a draft anywhere, that means *this* one
                                // isn't a draft, so return it as the latest
                                // non-draft entry.
                                if !found_draft {
                                    (found_draft, next)
                                } else {
                                    (found_draft, acc)
                                }
                            },
                        )
                        .1
                };

                entry_for_date.metadata.date
            }
        }
    }

    pub fn tags(&self) -> impl Iterator<Item = &TagName> {
        match self {
            Post::Single { metadata, .. } => metadata.tags.iter(),
            Post::Thread { metadata, .. } => metadata.tags.iter(),
        }
    }

    pub fn has_tag(&self, tag: &TagName) -> bool {
        match self {
            Post::Single { metadata, .. } => metadata.tags.contains(tag),
            Post::Thread { metadata, .. } => metadata.tags.contains(tag),
        }
    }

    pub fn is_entirely_draft(&self) -> bool {
        match self {
            Post::Single { metadata, .. } => metadata.draft,
            Post::Thread { entries, .. } => entries.iter().all(|entry| entry.metadata.draft),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThreadEntry {
    metadata: ThreadEntryMetadata,
    html_summary: String,
    html_content: String,
}

#[derive(Error, Debug)]
pub enum LoadPostError {
    #[error("failed to read content: {0}")]
    ReadContent(#[source] io::Error),

    #[error("post does not begin with frontmatter")]
    MissingFrontmatter,

    #[error("post frontmatter is malformed")]
    MalformedFrontmatter,

    #[error("failed to parse post frontmatter: {0}")]
    ParseFrontmatter(#[from] toml::de::Error),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostFrontmatter {
    #[serde(rename = "title")]
    md_title: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    tags: Vec<TagName>,
}

#[derive(Clone, Debug)]
pub struct SinglePostMetadata {
    pub md_title: String,
    pub draft: bool,
    pub tags: Vec<TagName>,
    pub date: NaiveDate,
}

impl SinglePostMetadata {
    fn split_for_thread(self) -> (ThreadMetadata, ThreadEntryMetadata) {
        let SinglePostMetadata {
            md_title,
            draft,
            tags,
            date,
        } = self;
        (
            ThreadMetadata { md_title, tags },
            ThreadEntryMetadata { draft, date },
        )
    }
}

#[derive(Clone, Debug)]
pub struct ThreadMetadata {
    pub md_title: String,
    pub tags: Vec<TagName>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadEntryMetadata {
    #[serde(default)]
    pub draft: bool,
    pub date: NaiveDate,
}

#[derive(Clone, Debug)]
pub struct Page {
    pub metadata: PageMetadata,
    pub html_content: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageMetadata {
    pub title: Option<String>,
}

#[derive(Error, Debug)]
pub enum LoadPageError {
    #[error("failed to read content: {0}")]
    ReadContent(#[source] io::Error),

    #[error("page does not begin with frontmatter")]
    MissingFrontmatter,

    #[error("page frontmatter is malformed")]
    MalformedFrontmatter,

    #[error("failed to parse page frontmatter: {0}")]
    ParseFrontmatter(#[from] toml::de::Error),
}

#[derive(Clone, Debug)]
pub struct Theme {
    theme_header: Arc<Markup>,
}

impl Theme {
    pub fn try_load(
        theme_set: SyntectThemeSet,
        light: &'static str,
        dark: &'static str,
    ) -> Result<Self, LoadThemeError> {
        use LoadThemeError::*;

        let light_css = css_for_theme_with_class_style(
            theme_set
                .themes
                .get(light)
                .ok_or_else(|| MissingTheme(light))?,
            ClassStyle::Spaced,
        )
        .map_err(GenerateThemeCss)?;
        let light_block = format!(":root {{ {light_css} }}");

        let dark_css = css_for_theme_with_class_style(
            theme_set
                .themes
                .get(dark)
                .ok_or_else(|| MissingTheme(dark))?,
            ClassStyle::Spaced,
        )
        .map_err(GenerateThemeCss)?;
        let dark_block = format!("@media(prefers-color-scheme: dark) {{ :root{{ {dark_css} }} }}");

        Ok(Self {
            theme_header: Arc::new(html! {
                (PreEscaped(light_block))
                (PreEscaped(dark_block))
            }),
        })
    }
}

#[derive(Error, Debug)]
pub enum LoadThemeError {
    #[error("failed to generate CSS for theme: {0}")]
    GenerateThemeCss(#[source] SyntectError),

    #[error("theme set does not contain a theme with name: {0}")]
    MissingTheme(&'static str),
}

impl Theme {
    pub fn theme_header(&self) -> &Markup {
        &self.theme_header
    }
}

impl FromRef<State> for Theme {
    fn from_ref(input: &State) -> Self {
        input.theme.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    show_drafts: bool,
}

impl Settings {
    pub fn show_drafts(&self) -> bool {
        self.show_drafts
    }
}

impl FromRef<State> for Settings {
    fn from_ref(input: &State) -> Self {
        input.settings.clone()
    }
}
