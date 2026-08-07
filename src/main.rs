mod argparse;
mod gui;
pub mod settings;
use std::{env::home_dir, process::exit, sync::mpsc::Receiver, thread, time::Duration};

use chilen_backend::music_lib::{self, ValueSeparators, covers};
use dirs::{cache_dir, data_dir};

use icu::locale::locale;
use log::{debug, error, info, warn};
use sys_locale::get_locale;

use crate::{argparse::parse_args, gui::THUMBNAIL_SIZE};

const APP_NAME: &str = "Chilen";
#[cfg(feature = "mpris")]
const APP_ID: &str = "dev.tpaau.Chilen";

fn handle_events(receiver: Receiver<chilen_backend::Event>) {
    // TODO: Cleaner way of doing this?
    loop {
        if gui::event_sender_initialized() {
            break;
        }
        thread::sleep(Duration::from_secs_f32(0.1));
    }
    loop {
        let event = match receiver.recv() {
            Ok(event) => event,
            Err(e) => {
                error!("Backend disconnected: {e}");
                return;
            }
        };
        gui::send_event(gui::Event::Backend(event));
    }
}

fn main() {
    thread::spawn(|| {
        let args = parse_args();

        let data_dir = match args.data_dir {
            Some(dir) => dir,
            None => match data_dir() {
                Some(mut dir) => {
                    dir.push(APP_NAME.to_lowercase());
                    dir
                }
                None => {
                    error!("Could not get the path to the data directory");
                    exit(1);
                }
            },
        };
        let cache_dir = match args.cache_dir {
            Some(dir) => dir,
            None => match cache_dir() {
                Some(mut dir) => {
                    dir.push(APP_NAME.to_lowercase());
                    dir
                }
                None => {
                    error!("Could not get the path to the cache directory");
                    exit(1);
                }
            },
        };
        let music_dir = match args.music_dir {
            Some(dir) => dir,
            None => match home_dir() {
                Some(mut dir) => {
                    dir.push("Music");
                    dir
                }
                None => {
                    error!("Could not get the path to the home directory");
                    exit(1);
                }
            },
        };

        let locale = match get_locale() {
            Some(l) => {
                debug!("Using locale \"{l}\" (system default)");
                l
            }
            None => {
                let l = String::from("en-US");
                warn!("Using fallback locale \"{l}\"");
                l
            }
        };

        // TODO: Some values in here should be user-controlled
        // That's the whole point of not hardcoding them on the backend
        let config = chilen_backend::Config {
            identity: APP_NAME.to_string(),
            #[cfg(feature = "mpris")]
            identifier: APP_ID.to_string(),
            data_dir,
            music_dir,
            cache_dir,
            locale: match locale.parse() {
                Ok(l) => l,
                Err(e) => {
                    error!("Couldn't parse the locale string: {e}");
                    locale!("en-US")
                }
            },
            library: music_lib::Config {
                value_separators: ValueSeparators::new(vec![
                    ",".to_string(),
                    "/".to_string(),
                    ";".to_string(),
                ]),
                covers: covers::Config {
                    format: covers::ImageFormat::Png,
                    thumbnail_resolution: THUMBNAIL_SIZE as u32,
                    cover_quality: covers::Quality::Default,
                },
            },
        };

        let receiver = match chilen_backend::init(config) {
            Ok(receiver) => receiver,
            Err(e) => panic!("Could not initialize the backend: {e}"),
        };

        handle_events(receiver);
    });

    match gui::start() {
        Ok(_) => info!("Main window closed, exiting"),
        Err(e) => {
            error!("GUI stopped unexpectedly: {e}");
            exit(1);
        }
    }
}
