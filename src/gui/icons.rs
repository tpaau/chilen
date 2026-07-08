#[cfg(windows)]
pub(super) const ICONS_FONT_BYTES: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\MaterialSymbolsRounded_Filled-Regular.ttf");
#[cfg(unix)]
pub(super) const ICONS_FONT_BYTES: &[u8] =
    include_bytes!("../../resources/fonts/MaterialSymbolsRounded_Filled-Regular.ttf");

const ICONS_FONT_NAME: &str = "Material Symbols Rounded Filled";

pub const HOME: u32 = 0xe88a;

pub const SIZE_SMALLER: u32 = 20;
pub const SIZE_SMALL: u32 = 26;
pub const SIZE_REGULAR: u32 = 32;
pub const SIZE_LARGE: u32 = 38;
pub const SIZE_LARGER: u32 = 44;

pub fn font() -> iced::Font {
    iced::Font {
        family: iced::font::Family::Name(ICONS_FONT_NAME),
        weight: iced::font::Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}
