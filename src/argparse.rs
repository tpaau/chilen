use std::path::PathBuf;

use clap::{Parser, ValueHint};
use env_logger::Builder;
use log::{LevelFilter, trace};

#[derive(Debug, Default, Clone, Copy, clap::ValueEnum)]
pub enum IndexingIntensity {
    #[default]
    Fast,
    Balanced,
    Lightweight,
}

impl std::fmt::Display for IndexingIntensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexingIntensity::Fast => write!(f, "fast"),
            IndexingIntensity::Balanced => write!(f, "balanced"),
            IndexingIntensity::Lightweight => write!(f, "lightweight"),
        }
    }
}

impl From<IndexingIntensity> for chilen_backend::music_lib::indexer::IndexingIntensity {
    fn from(value: IndexingIntensity) -> Self {
        match value {
            IndexingIntensity::Fast => Self::Fast,
            IndexingIntensity::Balanced => Self::Balanced,
            IndexingIntensity::Lightweight => Self::Lightweight,
        }
    }
}

#[derive(Parser)]
#[command(
    version,
    author = "tpaau <tpaau-17DB@tutamail.com>",
    help_template = "{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}
{all-args}{after-help}
"
)]
pub struct Args {
    /// Set the log filter level
    #[arg(long, short)]
    pub log_filter: Option<LevelFilter>,

    /// Override the music library directory
    #[cfg(feature = "dev-opts")]
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub music_dir_override: Option<PathBuf>,

    /// Override the cache directory
    #[cfg(feature = "dev-opts")]
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub cache_dir_override: Option<PathBuf>,

    /// Override the data directory
    #[cfg(feature = "dev-opts")]
    #[arg(long, value_hint = ValueHint::DirPath)]
    pub data_dir_override: Option<PathBuf>,

    #[arg(long, short, default_value_t = IndexingIntensity::default())]
    pub indexing_intensity: IndexingIntensity,

    #[arg(long, short, default_value_t = false)]
    pub rebuild_cache: bool,
}

pub fn parse_args() -> Args {
    let args = Args::parse();

    let foreign_module_filter = log::LevelFilter::Error;

    Builder::new()
        .filter_level(args.log_filter.unwrap_or(LevelFilter::Info))
        .filter_module("calloop", foreign_module_filter)
        .filter_module("cosmic_text", foreign_module_filter)
        .filter_module("iced_graphics", foreign_module_filter)
        .filter_module("iced_wgpu", foreign_module_filter)
        .filter_module("iced_winit", foreign_module_filter)
        .filter_module("lofty", foreign_module_filter)
        .filter_module("naga", foreign_module_filter)
        .filter_module("sctk", foreign_module_filter)
        .filter_module("tracing", foreign_module_filter)
        .filter_module("wgpu_core", foreign_module_filter)
        .filter_module("wgpu_hal", foreign_module_filter)
        .filter_module("winit", foreign_module_filter)
        .filter_module("zbus", foreign_module_filter)
        .filter_module("symphonia_core", foreign_module_filter)
        .filter_module("symphonia_bundle_mp3", foreign_module_filter)
        .filter_module("symphonia_bundle_flac", foreign_module_filter)
        .init();

    trace!("Finished parsing command line arguments");

    args
}
