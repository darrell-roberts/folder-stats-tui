use crate::folder_stats::FolderStat;
use std::collections::HashMap;

/// Sorting options for folders
#[derive(Debug, Copy, Clone, Default)]
pub enum SortBy {
    #[default]
    /// By total file sizes.
    FileSize,
    /// By total file counts.
    FileCount,
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
}

impl App {
    /// Height of a rendered item.
    pub const ITEM_HEIGHT: u16 = 4;

    /// Create a new [`App`].
    pub fn new() -> Self {
        Self {
            scanning: true,
            depth: 1,
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
}
