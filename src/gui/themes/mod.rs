mod graphite;
mod material;

use iced_m3::theme::{Mode, Palette, Theme};

pub const THEMES: &[Template] = &[graphite::TEMPLATE, material::TEMPLATE];

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
            black: self.dark.black(),
            mode,
        }
    }
}
