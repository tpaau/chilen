use iced_core::text::IntoFragment;
use iced_widget::Text;

#[cfg(all(windows, feature = "cjk_fonts"))]
pub(super) const BYTES_REGULAR_CJK: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\NotoSansCJK-Regular.ttc");
#[cfg(all(unix, feature = "cjk_fonts"))]
pub(super) const BYTES_REGULAR_CJK: &[u8] =
    include_bytes!("../../resources/fonts/NotoSansCJK-Regular.ttc");

#[cfg(all(windows, feature = "cjk_fonts"))]
pub(super) const BYTES_BOLD_CJK: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\NotoSansCJK-Bold.ttc");
#[cfg(all(unix, feature = "cjk_fonts"))]
pub(super) const BYTES_BOLD_CJK: &[u8] =
    include_bytes!("../../resources/fonts/NotoSansCJK-Bold.ttc");

#[cfg(unix)]
pub(super) const BYTES_REGULAR: &[u8] =
    include_bytes!("../../resources/fonts/NotoSans-Regular.ttf");
#[cfg(windows)]
pub(super) const BYTES_REGULAR: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\NotoSans-Regular.ttf");

#[cfg(unix)]
pub(super) const BYTES_BOLD: &[u8] = include_bytes!("../../resources/fonts/NotoSans-Bold.ttf");
#[cfg(windows)]
pub(super) const BYTES_BOLD: &[u8] = include_bytes!("..\\..\\resources\\fonts\\NotoSans-Bold.ttf");

pub(super) const NAME: &str = "Noto Sans";

pub const SIZE_SMALL: f32 = 14.0;
pub const SIZE_REGULAR: f32 = 16.0;
pub const SIZE_LARGE: f32 = 18.0;
pub const SIZE_LARGER: f32 = 20.0;

pub fn regular() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Normal,
        family: iced::font::Family::Name(NAME),
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

pub fn bold() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..regular()
    }
}

pub fn text<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    iced_widget::text(content).font(regular())
}

pub fn bold_text<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    iced_widget::text(content).font(bold())
}
