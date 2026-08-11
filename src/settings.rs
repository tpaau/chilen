use iced_m3::theme::Mode;

#[derive(Debug, Default, Clone)]
pub struct Settings {
    // TODO: Get dark mode preference from the host
    pub theme_mode: Mode,
    pub value_separator: String,
}

impl Settings {
    fn save(&self) {
        todo!()
    }

    pub fn load() -> Self {
        // TODO: Actually load the settings from here
        Self {
            theme_mode: Mode::Dark,
            value_separator: ", ".to_string(),
        }
    }

    pub fn set_theme_mode(&mut self, mode: Mode) {
        self.theme_mode = mode;
        self.save();
    }

    pub fn theme_mode(&self) -> Mode {
        self.theme_mode
    }
}
