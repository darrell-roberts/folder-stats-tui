use crate::{app::Filter, event::Event};
use ignore::{DirEntry, ParallelVisitor, ParallelVisitorBuilder, WalkBuilder, WalkState};
use log::error;
use std::{
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    sync::{mpsc::Sender, Arc},
};

struct MyParallelVisitor {
    root_path_bytes: Vec<u8>,
    sender: Sender<Event>,
    depth: usize,
}

impl MyParallelVisitor {
    fn truncate_root(&self, path: &str) -> String {
        let (_, path) = path.split_at(self.root_path_bytes.len());
        String::from(path)
    }
}

impl ParallelVisitor for MyParallelVisitor {
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
                        .skip(entry.depth().checked_sub(self.depth).unwrap_or(1))
                        .filter(|p| !p.is_symlink() && p.is_dir())
                        .flat_map(|p| p.as_os_str().to_str())
                        .take_while(|p| p.as_bytes().starts_with(&self.root_path_bytes))
                        .map(|name| (self.truncate_root(name), size))
                        .collect::<Vec<_>>();

                    if let Err(err) = self.sender.send(Event::FolderEvent(parents)) {
                        error!("Failed to emit folder events {err}");
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

struct MyVisitorBuilder {
    sender: Sender<Event>,
    depth: usize,
    root_path_bytes: Vec<u8>,
}

impl<'s> ParallelVisitorBuilder<'s> for MyVisitorBuilder {
    fn build(&mut self) -> Box<dyn ignore::ParallelVisitor + 's> {
        Box::new(MyParallelVisitor {
            sender: self.sender.clone(),
            depth: self.depth,
            root_path_bytes: self.root_path_bytes.clone(),
        })
    }
}

impl Drop for MyVisitorBuilder {
    fn drop(&mut self) {
        if let Err(err) = self.sender.send(Event::ScanComplete) {
            error!("Failed to emit scan complete {err}");
        }
    }
}

pub fn collect_stats(
    sender: Sender<Event>,
    depth: usize,
    root_path: PathBuf,
    filters: Arc<Vec<Filter>>,
) {
    let walker = WalkBuilder::new(&root_path)
        .filter_entry(move |entry| {
            (entry.file_type().map(|e| e.is_file()).unwrap_or(false)
                && check_filename_filter(entry, filters.clone())
                && check_file_extension_filter(entry, filters.clone()))
                || entry.file_type().map(|e| e.is_dir()).unwrap_or(false)
        })
        .build_parallel();

    let root_path_bytes = root_path.as_os_str().as_bytes().to_vec();

    let mut my_builder = MyVisitorBuilder {
        sender,
        depth,
        root_path_bytes,
    };

    walker.visit(&mut my_builder);
}

fn check_filename_filter(entry: &DirEntry, filters: Arc<Vec<Filter>>) -> bool {
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

fn check_file_extension_filter(entry: &DirEntry, filters: Arc<Vec<Filter>>) -> bool {
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
