use std::path::PathBuf;

use clap::{Parser, ValueHint};
use env_logger::Builder;
use log::{LevelFilter, trace};

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
    #[arg(long, short)]
    /// Set the log filter level
    pub log_filter: Option<LevelFilter>,

    #[arg(long, short, value_hint = ValueHint::DirPath)]
    pub music_dir: Option<PathBuf>,

    #[arg(long, short, value_hint = ValueHint::DirPath)]
    pub cache_dir: Option<PathBuf>,

    #[arg(long, short, value_hint = ValueHint::DirPath)]
    pub data_dir: Option<PathBuf>,

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
