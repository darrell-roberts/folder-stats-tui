//! Uses the `ignore` crate which is a type of folder traversal
//! that allows skpping folders via `.ignore` and `.gitignore` configuration
//! files found while traversing.
//!
use crate::{
    app::{Config, Filter, FolderStat},
    event::Event,
};
use ignore::{DirEntry, ParallelVisitor, ParallelVisitorBuilder, WalkBuilder, WalkState};
use log::error;
use std::{collections::HashMap, os::unix::ffi::OsStrExt, sync::mpsc::Sender};

/// Path visitor for each parallel thread worker.
struct MyParallelVisitor<'a> {
    root_path_bytes: &'a [u8],
    sender: Sender<Event>,
    depth: u8,
    results: HashMap<String, FolderStat>,
}

impl<'a> MyParallelVisitor<'a> {
    /// Convert the cannonical path into a relative path.
    fn truncate_root(&self, path: &str) -> String {
        let (_, path) = path.split_at(self.root_path_bytes.len());
        String::from(path)
    }
}

impl<'a> ParallelVisitor for MyParallelVisitor<'a> {
    /// Visit each directory entry.
    fn visit(&mut self, result: Result<DirEntry, ignore::Error>) -> WalkState {
        match result {
            Ok(entry) => {
                if entry.path().is_dir() {
                    let folder_name = entry
                        .path()
                        .to_str()
                        .map(|p| self.truncate_root(p))
                        .unwrap_or_else(|| {
                            self.truncate_root(entry.path().to_string_lossy().as_ref())
                        });
                    if let Err(err) = self.sender.send(Event::Progress(folder_name)) {
                        error!("Failed to emit folder name: {err}");
                    }
                } else if let Ok(size) = entry.metadata().map(|md| md.len()) {
                    let parents = entry
                        .path()
                        .ancestors()
                        .skip(entry.depth().checked_sub(self.depth as usize).unwrap_or(1))
                        .filter(|p| !p.is_symlink() && p.is_dir())
                        .flat_map(|p| p.as_os_str().to_str())
                        .take_while(|p| p.as_bytes().starts_with(&self.root_path_bytes));

                    for parent in parents {
                        self.results
                            .entry(self.truncate_root(parent))
                            .and_modify(|fs: &mut FolderStat| {
                                fs.size += size;
                                fs.files += 1;
                            })
                            .or_insert(FolderStat { size, files: 1 });
                    }
                }
                WalkState::Continue
            }
            Err(err) => {
                error!("Failed to walk {err}");
                WalkState::Quit
            }
        }
    }
}

impl<'a> Drop for MyParallelVisitor<'a> {
    fn drop(&mut self) {
        let results = std::mem::take(&mut self.results);
        if let Err(err) = self.sender.send(Event::FolderEvent(results)) {
            error!("Failed to emit folder events {err}");
        }
    }
}

/// Parallel visitor builder.
struct MyVisitorBuilder<'a> {
    sender: Sender<Event>,
    depth: u8,
    root_path_bytes: &'a [u8],
}

impl<'a> ParallelVisitorBuilder<'a> for MyVisitorBuilder<'a> {
    /// Build an [`ignore::ParallelVisitor`].
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 'a> {
        Box::new(MyParallelVisitor {
            sender: self.sender.clone(),
            depth: self.depth,
            root_path_bytes: &self.root_path_bytes,
            results: HashMap::new(),
        })
    }
}

impl<'a> Drop for MyVisitorBuilder<'a> {
    fn drop(&mut self) {
        if let Err(err) = self.sender.send(Event::ScanComplete) {
            error!("Failed to emit scan complete {err}");
        }
    }
}

/// Spawn a thread that will configure and launch the ignore parallel walker. Each
/// visitor will collect it's results and then emit them when dropped. The builder
/// emits a traversal completed event when it is dropped.
pub fn collect_stats(sender: Sender<Event>, config: Config) {
    std::thread::spawn(move || {
        let c = config.clone();
        let walker = WalkBuilder::new(&config.root_path)
            .filter_entry(move |entry| {
                (entry.file_type().map(|e| e.is_file()).unwrap_or(false)
                    && check_filename_filter(entry, &c.filters)
                    && check_file_extension_filter(entry, &c.filters))
                    || entry.file_type().map(|e| e.is_dir()).unwrap_or(false)
            })
            .ignore(!config.no_ignores)
            .hidden(!config.show_hidden)
            .git_ignore(!config.no_ignores)
            .build_parallel();

        let root_path_bytes = config.root_path.as_os_str().as_bytes().to_vec();

        let mut my_builder = MyVisitorBuilder {
            sender,
            depth: config.depth,
            root_path_bytes: &root_path_bytes,
        };

        walker.visit(&mut my_builder);
    });
}

fn check_filename_filter(entry: &DirEntry, filters: &[Filter]) -> bool {
    let filename_filter = || filters.iter().filter(|f| matches!(f, Filter::FileName(_)));
    // Check filename filters.
    if filename_filter().next().is_some() {
        match entry.file_name().to_str() {
            Some(filename) => filename_filter().any(|f| f.contains(filename)),
            None => false,
        }
    } else {
        true
    }
}

fn check_file_extension_filter(entry: &DirEntry, filters: &[Filter]) -> bool {
    let extension_filter = || filters.iter().filter(|f| matches!(f, Filter::Extension(_)));
    // Check extensions filters.
    if extension_filter().next().is_some() {
        match entry.path().extension().and_then(|s| s.to_str()) {
            Some(extension) => extension_filter().any(|f| f.contains(extension)),
            None => false,
        }
    } else {
        true
    }
}
