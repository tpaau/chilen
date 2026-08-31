#[cfg(all(windows, feature = "cjk_fonts"))]
pub(super) const BYTES_REGULAR: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\NotoSansCJK-Regular.ttc");
#[cfg(all(unix, feature = "cjk_fonts"))]
pub(super) const BYTES_REGULAR: &[u8] =
    include_bytes!("../../resources/fonts/NotoSansCJK-Regular.ttc");

#[cfg(all(windows, feature = "cjk_fonts"))]
pub(super) const BYTES_BOLD: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\NotoSansCJK-Bold.ttc");
#[cfg(all(unix, feature = "cjk_fonts"))]
pub(super) const BYTES_BOLD: &[u8] = include_bytes!("../../resources/fonts/NotoSansCJK-Bold.ttc");

#[cfg(all(unix, not(feature = "cjk_fonts")))]
pub(super) const BYTES_REGULAR: &[u8] =
    include_bytes!("../../resources/fonts/NotoSans-Regular.ttf");
#[cfg(all(windows, not(feature = "cjk_fonts")))]
pub(super) const BYTES_REGULAR: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\NotoSans-Regular.ttf");

#[cfg(all(unix, not(feature = "cjk_fonts")))]
pub(super) const BYTES_BOLD: &[u8] = include_bytes!("../../resources/fonts/NotoSans-Bold.ttf");
#[cfg(all(windows, not(feature = "cjk_fonts")))]
pub(super) const BYTES_BOLD: &[u8] = include_bytes!("..\\..\\resources\\fonts\\NotoSans-Bold.ttf");

#[cfg(feature = "cjk_fonts")]
pub(super) const NAME: &str = "Noto Sans CJK";
#[cfg(not(feature = "cjk_fonts"))]
pub(super) const NAME: &str = "Noto Sans";

pub const SIZE_SMALL: f32 = 14.0;
pub const SIZE_REGULAR: f32 = 16.0;
pub const SIZE_LARGE: f32 = 18.0;
pub const SIZE_LARGER: f32 = 20.0;

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
