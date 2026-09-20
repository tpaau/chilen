use std::{fs::File, io::Write, path::PathBuf, sync::LazyLock};

use iced_m3::theme::Mode;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::gui::{Chilen, dialog::Dialog, themes::THEMES};

static SETTINGS_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut data_dir = crate::DATA_DIR
        .clone()
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .clone();
    data_dir.push("settings.json");
    data_dir
});

#[derive(Serialize, Deserialize)]
struct StoredSettings {
    theme_name: String,
    theme_dark_mode: bool,
    value_separator: String,
    show_lyrics_errors: bool,
}

impl From<Settings> for StoredSettings {
    fn from(value: Settings) -> Self {
        let theme_dark_mode = match value.theme_mode {
            Mode::Light => false,
            Mode::Dark => true,
        };
        Self {
            theme_name: value.theme_name,
            theme_dark_mode,
            value_separator: value.value_separator,
            show_lyrics_errors: value.show_lyrics_errors,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub theme_name: String,
    // TODO: Get dark mode preference from the host
    pub theme_mode: Mode,
    pub value_separator: String,
    /// Whether Chilen should display errors when it detects lyrics are synchronized but malformed.
    pub show_lyrics_errors: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_name: THEMES[0].name.to_string(),
            theme_mode: Mode::default(),
            value_separator: ", ".to_string(),
            show_lyrics_errors: true,
        }
    }
}

impl From<StoredSettings> for Settings {
    fn from(value: StoredSettings) -> Self {
        let theme_mode = match value.theme_dark_mode {
            true => Mode::Dark,
            false => Mode::Light,
        };
        Self {
            theme_name: value.theme_name,
            theme_mode,
            value_separator: value.value_separator,
            show_lyrics_errors: value.show_lyrics_errors,
        }
    }
}

impl Settings {
    fn save(self) -> Result<(), String> {
        let settings: StoredSettings = self.into();
        let data = match serde_json::to_string_pretty(&settings) {
            Ok(data) => data,
            Err(e) => return Err(e.to_string()),
        };

        let mut handle = match File::create(SETTINGS_FILE.clone()) {
            Ok(handle) => handle,
            Err(e) => return Err(e.to_string()),
        };

        if let Err(e) = handle.write_all(data.as_bytes()) {
            return Err(e.to_string());
        }

        Ok(())
    }

    pub fn load() -> Self {
        // TODO: Actually load the settings from here
        Self::default()
    }

    pub fn set_theme_mode(&mut self, mode: Mode) {
        self.theme_mode = mode;
    }
}

pub fn save(state: &mut Chilen) {
    info!("Saving settings state to disk");
    if let Err(e) = state.settings.clone().save() {
        let msg = format!("Failed to save settings to disk: {e}");
        error!("{}", msg.clone());
        state.dialog = Dialog::Error(msg)
    }
}
