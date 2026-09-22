mod breathe;
mod bubblegum;
mod graphite;
mod mahogany;
mod material;

use iced_m3::theme::{Mode, Palette, Theme};

// Themes listed here appear in the same order in the settings. The first theme is the default.
pub const THEMES: &[Template] = &[
    breathe::TEMPLATE,
    graphite::TEMPLATE,
    mahogany::TEMPLATE,
    material::TEMPLATE,
    bubblegum::TEMPLATE,
];

#[derive(Clone)]
pub struct Template<'a> {
    pub name: &'a str,
    pub dark: Palette,
    pub light: Palette,
}

impl<'a> Template<'a> {
    pub fn into_theme(self, mode: Mode) -> Theme {
        Theme {
            dark: self.dark,
            light: self.light,
            mode,
        }
    }
}
