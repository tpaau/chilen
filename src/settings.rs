use std::{fs::File, io::Write, path::PathBuf, sync::LazyLock};

use iced_m3::theme::Mode;
use serde::{Deserialize, Serialize};

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
            theme_dark_mode,
            value_separator: value.value_separator,
            show_lyrics_errors: value.show_lyrics_errors,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Settings {
    // TODO: Get dark mode preference from the host
    pub theme_mode: Mode,
    pub value_separator: String,
    /// Whether Chilen should display errors when it detects lyrics are synchronized but malformed.
    pub show_lyrics_errors: bool,
}

impl From<StoredSettings> for Settings {
    fn from(value: StoredSettings) -> Self {
        let theme_mode = match value.theme_dark_mode {
            true => Mode::Dark,
            false => Mode::Light,
        };
        Self {
            theme_mode,
            value_separator: value.value_separator,
            show_lyrics_errors: value.show_lyrics_errors,
        }
    }
}

impl Settings {
    pub fn save(self) -> Result<(), String> {
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
        Self {
            theme_mode: Mode::Dark,
            value_separator: ", ".to_string(),
            show_lyrics_errors: true,
        }
    }

    pub fn set_theme_mode(&mut self, mode: Mode) {
        self.theme_mode = mode;
    }
}
