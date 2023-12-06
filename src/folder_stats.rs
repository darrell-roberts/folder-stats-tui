use crate::event::Event;
use log::error;
use std::{
    collections::HashMap,
    env::args,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
    thread,
};
use walkdir::WalkDir;

#[derive(Debug, Copy, Clone)]
pub struct FolderStat {
    pub size: u64,
    pub files: usize,
}

/// Recusively scan folders from provided location or current location.
/// Emit progress and final scan results.
pub fn collect_folder_stats(sender: Sender<Event>, depth: usize) -> anyhow::Result<()> {
    let root_path = Path::new(&args().nth(1).unwrap_or_else(|| ".".into())).canonicalize()?;
    thread::spawn(move || {
        scan_folders(sender, root_path, depth);
    });
    Ok(())
}

fn scan_folders(sender: Sender<Event>, root_path: PathBuf, depth: usize) {
    let mut folder_sizes = HashMap::new();
    let root_path_bytes = root_path.as_os_str().as_bytes();
    let walker = WalkDir::new(&root_path)
        .into_iter()
        .flat_map(|f| f.ok())
        .filter(|entry| !entry.path_is_symlink());

    let truncate_root = |p: &str| {
        let (_, path) = p.split_at(root_path_bytes.len());
        String::from(path)
    };

    for entry in walker {
        // Emit status.
        if entry.path().is_dir() {
            let folder_name = entry
                .path()
                .to_str()
                .map(truncate_root)
                .unwrap_or_else(|| truncate_root(entry.path().to_string_lossy().as_ref()));
            if let Err(err) = sender.send(Event::Progress(folder_name)) {
                error!("Failed to emit folder name: {err}");
            }
        } else if let Ok(size) = entry.metadata().map(|md| md.len()) {
            let parents = entry
                .path()
                .ancestors()
                .skip(entry.depth().checked_sub(depth).unwrap_or(1))
                .filter(|p| !p.is_symlink() && p.is_dir())
                .flat_map(|p| p.as_os_str().to_str())
                .take_while(|p| p.as_bytes().starts_with(root_path_bytes))
                .map(truncate_root);

            // update stats for all ancestor folders.
            for parent_folder in parents {
                folder_sizes
                    .entry(parent_folder)
                    .and_modify(|fs: &mut FolderStat| {
                        fs.size += size;
                        fs.files += 1;
                    })
                    .or_insert(FolderStat { size, files: 1 });
            }
        }
    }

    if let Err(err) = sender.send(Event::ScanComplete(folder_sizes)) {
        error!("Failed to emit results {err}");
    }
}
