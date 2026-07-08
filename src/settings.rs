use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    // TODO: Get dark mode preference from the host
    dark_theme: bool,
}

impl Settings {
    fn save(&self) {
        todo!()
    }

    pub fn load() -> Self {
        // TODO: Actually load the settings from here
        Self { dark_theme: true }
    }

    pub fn set_dark_theme(&mut self, dark_theme: bool) {
        self.dark_theme = dark_theme;
        self.save();
    }

    pub fn dark_theme(&self) -> bool {
        self.dark_theme
    }
}
