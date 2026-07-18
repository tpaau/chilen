use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    // TODO: Get dark mode preference from the host
    dark_mode: bool,
}

impl Settings {
    // TODO: Add error handling here for when the indexer is running (too many open files)
    fn save(&self) {
        todo!()
    }

    pub fn load() -> Self {
        // TODO: Actually load the settings from here
        Self { dark_mode: true }
    }

    pub fn set_dark_theme(&mut self, dark_mode: bool) {
        self.dark_mode = dark_mode;
        self.save();
    }

    pub fn dark_mode(&self) -> bool {
        self.dark_mode
    }
}
