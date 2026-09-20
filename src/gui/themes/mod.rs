mod graphite;
mod material;

use iced_m3::theme::Palette;

pub const THEMES: &[Template] = &[graphite::TEMPLATE, material::TEMPLATE];

pub struct Template<'a> {
    pub name: &'a str,
    pub dark: Palette,
    pub light: Palette,
}
