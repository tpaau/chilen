mod argparse;
mod gui;
pub mod settings;
use std::{env::home_dir, process::exit, sync::mpsc::Receiver, thread, time::Duration};

use chilen_backend::Config;
use dirs::{cache_dir, data_dir};

use log::{error, info};

use crate::argparse::parse_args;

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

        let config = Config {
            data_dir,
            music_dir,
            cache_dir,
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
