use crate::app::Filter;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short = 'p', long = "path", default_value = ".")]
    pub root_path: PathBuf,

    #[arg(short, long, default_value_t = 1)]
    pub depth: usize,

    #[arg(short = 'f', long, value_parser = |s: &str| {
        Ok::<_, std::convert::Infallible>(Filter::FileName(s.to_string()))
    })]
    pub filter: Option<Filter>,

    #[arg(short = 'e', long = "extension", value_parser = |s: &str| {
        Ok::<_, std::convert::Infallible>(Filter::Extension(s.to_string()))
    })]
    pub extension_filter: Option<Filter>,

    #[arg(short = 'i', long, default_value_t = false)]
    pub no_ignores: bool,
}

impl Args {
    pub fn filters(self) -> Vec<Filter> {
        self.filter
            .into_iter()
            .chain(self.extension_filter)
            .collect()
    }
}
