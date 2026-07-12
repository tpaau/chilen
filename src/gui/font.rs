#[cfg(windows)]
pub(super) const BYTES_REGULAR: &[u8] =
    include_bytes!("..\\..\\resources\\Roboto\\NotoSans-Regular.ttf");
#[cfg(unix)]
pub(super) const BYTES_REGULAR: &[u8] =
    include_bytes!("../../resources/fonts/NotoSans-Regular.ttf");
#[cfg(windows)]
pub(super) const BYTES_BOLD: &[u8] = include_bytes!("..\\..\\resources\\Roboto\\NotoSans-Bold.ttf");
#[cfg(unix)]
pub(super) const BYTES_BOLD: &[u8] = include_bytes!("../../resources/fonts/NotoSans-Bold.ttf");

pub const SIZE_SMALLER: u32 = 12;
pub const SIZE_SMALL: u32 = 14;
pub const SIZE_REGULAR: u32 = 16;
pub const SIZE_LARGE: u32 = 18;
pub const SIZE_LARGER: u32 = 18;

const NAME: &str = "Noto Sans";

pub fn font() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Normal,
        family: iced::font::Family::Name(NAME),
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

pub fn font_bold() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..font()
    }
}
