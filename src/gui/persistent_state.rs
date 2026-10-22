use std::{
    fs::{File, read_to_string},
    io::Write,
    path::PathBuf,
    sync::LazyLock,
};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::gui::playback_view;

static STATE_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data_dir = crate::DATA_DIR
        .clone()
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    data_dir.push("state.json");
    data_dir
});

#[derive(Default, Clone)]
pub struct State {
    pub sidebar_opened: bool,
    pub playback_tab: playback_view::Tab,
}

#[derive(Serialize, Deserialize)]
struct StoredState {
    sidebar_opened: bool,
    playback_tab: playback_view::Tab,
}

impl From<StoredState> for State {
    fn from(value: StoredState) -> Self {
        Self {
            sidebar_opened: value.sidebar_opened,
            playback_tab: value.playback_tab,
        }
    }
}

impl From<State> for StoredState {
    fn from(value: State) -> Self {
        Self {
            sidebar_opened: value.sidebar_opened,
            playback_tab: value.playback_tab,
        }
    }
}

impl State {
    pub(super) fn save(self) -> Result<(), String> {
        let state: StoredState = self.into();
        let data = match serde_json::to_string_pretty(&state) {
            Ok(data) => data,
            Err(e) => return Err(e.to_string()),
        };

        let mut handle = match File::create(STATE_FILE.clone()) {
            Ok(handle) => handle,
            Err(e) => return Err(e.to_string()),
        };

        if let Err(e) = handle.write_all(data.as_bytes()) {
            return Err(e.to_string());
        }

        Ok(())
    }

    pub fn load() -> Result<Self, String> {
        let data = match read_to_string(STATE_FILE.clone()) {
            Ok(data) => data,
            Err(e) => {
                let msg = format!("Failed to load app state: {e}");
                error!("{}", msg.clone());
                msg
            }
        };

        let stored_state: StoredState = match serde_json::from_str(&data) {
            Ok(state) => state,
            Err(e) => {
                let msg = format!("Failed to load app state: {e}");
                error!("{}", msg.clone());
                return Err(msg);
            }
        };

        info!("App state loaded");

        Ok(stored_state.into())
    }
}
