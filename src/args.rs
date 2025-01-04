use crate::app::Filter;
use clap::Parser;
use std::path::PathBuf;

/// Command line arguments.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(
        short = 'p',
        long = "path",
        default_value = ".",
        help = "Folder to scan.",
        id = "PATH"
    )]
    pub root_path: PathBuf,

    #[arg(short, long, default_value_t = 1, help = "Folder depth to see in Tui")]
    pub depth: u8,

    #[arg(short = 'f', long, value_parser = |s: &str| {
        Ok::<_, std::convert::Infallible>(Filter::FileName(s.to_string()))
    }, id = "FILENAME_FILTER", help = "Filter files that contain text")]
    pub filter: Vec<Filter>,

    #[arg(short = 'e', long = "extension", value_parser = |s: &str| {
        Ok::<_, std::convert::Infallible>(Filter::Extension(s.to_string()))
    }, help = "Filter by file extension. Ex: -e rs")]
    pub extension_filter: Vec<Filter>,

    #[arg(
        short = 'i',
        long,
        default_value_t = false,
        help = "Disable .ignore, .gitignore filtering"
    )]
    pub no_ignores: bool,

    #[arg(long, default_value_t = false, help = "Disable hidden file filtering")]
    pub show_hidden: bool,
}

impl Args {
    /// Consume args and produce [`Vec<Filter>`].
    pub fn filters(self) -> Vec<Filter> {
        self.filter
            .into_iter()
            .chain(self.extension_filter)
            .collect()
    }
}
