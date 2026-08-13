use std::sync::LazyLock;

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

const FILLED_ICONS_FONT_NAME: &str = "Material Symbols Rounded Filled";
const OUTLINED_ICONS_FONT_NAME: &str = "Material Symbols Rounded";

pub static MORE_HORIZ: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5d3).unwrap());
pub static ADD: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe145).unwrap());
pub static CLOSE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5cd).unwrap());
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
pub static PLAYLIST_PLAY: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe05f).unwrap());
pub static ARROW_BACK: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5c4).unwrap());

pub const SIZE_SMALLER: u32 = 16;
pub const SIZE_SMALL: u32 = 20;
pub const SIZE_REGULAR: u32 = 24;
pub const SIZE_LARGE: u32 = 28;
pub const SIZE_LARGER: u32 = 32;

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
