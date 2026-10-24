use std::sync::LazyLock;

use iced::Pixels;
use iced_core::text::IntoFragment;
use iced_widget::Text;

#[cfg(unix)]
pub(super) const FILLED_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("../../resources/fonts/MaterialSymbolsRounded_Filled-Regular.ttf");
#[cfg(windows)]
pub(super) const FILLED_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\MaterialSymbolsRounded_Filled-Regular.ttf");

#[cfg(unix)]
pub(super) const OUTLINED_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("../../resources/fonts/MaterialSymbolsRounded-Regular.ttf");
#[cfg(windows)]
pub(super) const OUTLINED_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\MaterialSymbolsRounded-Regular.ttf");

pub(super) const FILLED_ICONS_FONT_NAME: &str = "Material Symbols Rounded Filled";
pub(super) const OUTLINED_ICONS_FONT_NAME: &str = "Material Symbols Rounded";

pub static MORE_HORIZ: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5d3).unwrap());
pub static PLAYLIST_ADD: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe03b).unwrap());
pub static UPLOAD_FILE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe9fc).unwrap());
pub static PLAY_ARROW: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe037).unwrap());
pub static SHUFFLE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe043).unwrap());
pub static UPLOAD: LazyLock<char> = LazyLock::new(|| char::from_u32(0xf09b).unwrap());
pub static EDIT: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe3c9).unwrap());
pub static DELETE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe872).unwrap());
pub static IMAGE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe3f4).unwrap());
pub static ADD_TO_QUEUE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe05c).unwrap());
pub static ARTIST: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe01a).unwrap());
pub static ALBUM: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe019).unwrap());
pub static MUSIC_NOTE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe405).unwrap());
pub static GENRES: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe022).unwrap());
pub static ARROW_BACK: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5c4).unwrap());
pub static SEARCH: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe8b6).unwrap());
pub static SETTINGS: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe8b8).unwrap());
pub static INFO: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe88e).unwrap());
pub static SKIP_PREVIOUS: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe045).unwrap());
pub static SKIP_NEXT: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe044).unwrap());
pub static PAUSE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe034).unwrap());
pub static STOP: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe047).unwrap());
pub static REPEAT: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe040).unwrap());
pub static REPEAT_ONE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe041).unwrap());
pub static LYRICS: LazyLock<char> = LazyLock::new(|| char::from_u32(0xec0b).unwrap());
pub static QUEUE_MUSIC: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe03d).unwrap());
pub static ERROR: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe000).unwrap());
pub static REFRESH: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5d5).unwrap());
pub static PALETTE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe40a).unwrap());
pub static LIBRARY_MUSIC: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe030).unwrap());
pub static CHECK: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5ca).unwrap());
pub static PERSON: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe7fd).unwrap());
pub static RESET_SETTINGS: LazyLock<char> = LazyLock::new(|| char::from_u32(0xf47f).unwrap());
pub static SNOWFLAKE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xed5b).unwrap());
pub static SCHEDULE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe8b5).unwrap());
pub static COMMIT: LazyLock<char> = LazyLock::new(|| char::from_u32(0xeaf5).unwrap());
pub static CODE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe86f).unwrap());
pub static LANGUAGE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe894).unwrap());
pub static MAIL: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe158).unwrap());
pub static HOME: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe88a).unwrap());
pub static ARROW_FORWARD_IOS: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5e1).unwrap());
pub static EQUALIZER: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe01d).unwrap());
pub static TRENDING_UP: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe8e5).unwrap());
pub static FAVORITE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe87d).unwrap());

pub const SIZE_SMALLER: f32 = 16.0;
pub const SIZE_SMALL: f32 = 20.0;
pub const SIZE_REGULAR: f32 = 24.0;
pub const SIZE_LARGE: f32 = 28.0;
pub const SIZE_LARGER: f32 = 32.0;

pub fn filled() -> iced::Font {
    iced::Font {
        family: iced::font::Family::Name(FILLED_ICONS_FONT_NAME),
        weight: iced::font::Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

pub fn outlined() -> iced::Font {
    iced::Font {
        family: iced::font::Family::Name(OUTLINED_ICONS_FONT_NAME),
        weight: iced::font::Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

pub fn icon_filled<'a, P>(icon: impl IntoFragment<'a>, size: P) -> Text<'a>
where
    P: Into<Pixels> + Copy,
{
    iced_m3::widget::icon(icon, size).font(filled())
}

pub fn icon_outlined<'a, P>(icon: impl IntoFragment<'a>, size: P) -> Text<'a>
where
    P: Into<Pixels> + Copy,
{
    iced_m3::widget::icon(icon, size).font(outlined())
}
