use crate::args::Args;
use std::{borrow::Cow, collections::HashMap, path::PathBuf, sync::Arc};

/// Sorting options for folders
#[derive(Debug, Copy, Clone, Default)]
pub enum SortBy {
    #[default]
    /// By total file sizes.
    FileSize,
    /// By total file counts.
    FileCount,
}

/// Filters to apply to scan.
#[derive(Debug, Clone)]
pub enum Filter {
    /// File name extension filter.
    Extension(String),
    /// File name filter.
    FileName(String),
}

impl Filter {
    pub fn contains(&self, test: &str) -> bool {
        match self {
            Filter::Extension(s) => test.contains(s),
            Filter::FileName(s) => test.contains(s),
        }
    }
}

/// Statistics for a folder.
#[derive(Debug, Copy, Clone)]
pub struct FolderStat {
    /// Recursive total file sizes.
    pub size: u64,
    /// Recursive total file count.
    pub files: usize,
}

/// Application configuration sourced
/// from command line argument options.
#[derive(Debug, Default)]
pub struct Config {
    /// Path to scan.
    pub root_path: PathBuf,
    /// Filters for scan.
    pub filters: Vec<Filter>,
    /// Disable ignores support
    pub no_ignores: bool,
    /// Initial depth to render on first scan.
    pub depth: usize,
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let root_path = args.root_path.canonicalize()?;

        Ok(Self {
            root_path,
            no_ignores: args.no_ignores,
            depth: args.depth,
            filters: args.filters(),
        })
    }
}

/// Application State.
#[derive(Debug, Default)]
pub struct App {
    /// Name of the folder being scanned.
    pub folder_name: String,
    /// Result of scanned folders with folder stats.
    pub scan_result: Vec<(String, FolderStat)>,
    /// Flag to terminate program.
    pub should_quit: bool,
    /// Index for where scrolling is set.
    pub scroll_state: usize,
    /// Maximum scroll index.
    pub max_scroll: usize,
    /// True if we are scanning folders.
    pub scanning: bool,
    /// Depth to report on.
    pub depth: usize,
    /// Sorting of folders.
    pub sort: SortBy,
    /// Content height.
    pub content_height: u16,
    /// Folder events emitted by walker.
    pub folder_events: HashMap<String, FolderStat>,
    /// Initial configuration from program launch.
    pub config: Arc<Config>,
    /// Show help popup.
    pub show_help: bool,
}

impl App {
    /// Height of a rendered item.
    pub const ITEM_HEIGHT: u16 = 4;

    /// Create a new [`App`].
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            scanning: true,
            depth: config.depth,
            config,
            ..Default::default()
        }
    }

    /// Update scan progress with folder being scanned.
    pub fn update_progress(&mut self, folder_name: String) {
        self.folder_name = folder_name;
    }

    /// Update state with scan results.
    pub fn update_scan_result(&mut self, result: HashMap<String, FolderStat>) {
        let mut file_rows = result.into_iter().collect::<Vec<_>>();
        file_rows.sort_unstable_by_key(|(_, v)| v.size);
        self.sort = SortBy::FileSize;
        self.scan_result = file_rows;
        self.scroll_state = 0;
        self.compute_max_scroll()
    }

    /// Signal program termination.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Scroll up.
    pub fn scroll_up(&mut self, val: usize) {
        let up = self.scroll_state.checked_sub(val).unwrap_or_default();
        self.scroll_state = up;
    }

    /// Scroll down.
    pub fn scroll_down(&mut self, val: usize) {
        let down = self.scroll_state + val;
        if down <= self.max_scroll {
            self.scroll_state = down;
        } else {
            self.scroll_state = self.max_scroll;
        }
    }

    /// Compute what the maximum scroll index should be based
    /// on the content height and the total number of results.
    pub fn compute_max_scroll(&mut self) {
        if self
            .content_height
            .checked_div(Self::ITEM_HEIGHT)
            .unwrap_or(self.content_height) as usize
            > self.scan_result.len()
        {
            self.max_scroll = 0;
        } else {
            self.max_scroll = self
                .content_height
                .checked_div(Self::ITEM_HEIGHT)
                .and_then(|sub| self.scan_result.len().checked_sub(sub as usize))
                .unwrap_or(self.scan_result.len());
        }
    }

    /// Compute the number of scroll state units for a full page.
    pub fn compute_scroll_page(&self) -> usize {
        (self.content_height / Self::ITEM_HEIGHT) as usize
    }

    pub fn root_folder(&self) -> Cow<str> {
        self.config
            .root_path
            .as_path()
            .to_str()
            .map(Cow::Borrowed)
            .unwrap_or(self.config.root_path.to_string_lossy())
    }
}
