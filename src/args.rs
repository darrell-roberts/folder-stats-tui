use std::path::PathBuf;

use clap::Parser;

use crate::app::Filter;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long, default_value = ".")]
    pub root_path: PathBuf,

    #[arg(short, long, default_value_t = 1)]
    pub depth: usize,

    #[arg(short, long)]
    pub filter: Option<String>,

    #[arg(short, long)]
    pub extension_filter: Option<String>,
}

impl Args {
    pub fn filters(self) -> Vec<Filter> {
        self.filter
            .into_iter()
            .map(Filter::FileName)
            .chain(self.extension_filter.into_iter().map(Filter::Extension))
            .collect()
    }
}
