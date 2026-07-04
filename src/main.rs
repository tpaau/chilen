mod argparse;
mod gui;
pub mod music_lib;
mod playback;
#[cfg(test)]
mod tests;

use std::{env::home_dir, process::exit};

use dirs::{cache_dir, data_dir};

use log::error;

use crate::{
    argparse::parse_args,
    music_lib::{covers::LoadMode, set_dirs},
};

fn main() {
    let args = parse_args();

    let data_dir = match args.data_dir {
        Some(dir) => dir,
        None => match data_dir() {
            Some(dir) => dir,
            None => {
                error!("Could not get the path to the data directory");
                exit(1);
            }
        },
    };
    let cache_dir = match args.cache_dir {
        Some(dir) => dir,
        None => match cache_dir() {
            Some(dir) => dir,
            None => {
                error!("Could not get the path to the data directory");
                exit(1);
            }
        },
    };
    let music_dir = match args.music_dir {
        Some(dir) => dir,
        None => {
            let mut dir = match home_dir() {
                Some(home) => home,
                None => {
                    error!("Could not get the path to the home directory");
                    exit(1);
                }
            };
            dir.push("Music");
            dir
        }
    };
    set_dirs(data_dir, cache_dir, music_dir).unwrap();
    music_lib::state::load(LoadMode::Load).unwrap();
    playback::init(
        #[cfg(feature = "mpris")]
        "Chilen".to_string(),
        #[cfg(feature = "mpris")]
        "dev.tpaau.Chilen".to_string(),
    );

    gui::start().unwrap();
}
