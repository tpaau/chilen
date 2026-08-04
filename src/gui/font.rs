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

pub const SIZE_SMALLER: f32 = 12.0;
pub const SIZE_SMALL: f32 = 14.0;
pub const SIZE_REGULAR: f32 = 16.0;
pub const SIZE_LARGE: f32 = 18.0;
pub const SIZE_LARGER: f32 = 20.0;

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
