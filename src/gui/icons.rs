use std::sync::LazyLock;

#[cfg(windows)]
pub(super) const ICONS_FONT_BYTES: &[u8] =
    include_bytes!("..\\..\\resources\\fonts\\MaterialSymbolsRounded_Filled-Regular.ttf");
#[cfg(unix)]
pub(super) const ICONS_FONT_BYTES: &[u8] =
    include_bytes!("../../resources/fonts/MaterialSymbolsRounded_Filled-Regular.ttf");

const ICONS_FONT_NAME: &str = "Material Symbols Rounded Filled";

pub static MORE_HORIZ: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5d3).unwrap());
pub static ADD: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe145).unwrap());
pub static CLOSE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe5cd).unwrap());
pub static PLAYLIST_ADD: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe03b).unwrap());
pub static UPLOAD_FILE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe9fc).unwrap());
pub static PLAY_ARROW: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe037).unwrap());
pub static SHUFFLE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe043).unwrap());
pub static UPLOAD: LazyLock<char> = LazyLock::new(|| char::from_u32(0xf09b).unwrap());
pub static CONTENT_COPY: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe14d).unwrap());
pub static EDIT: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe3c9).unwrap());
pub static DELETE: LazyLock<char> = LazyLock::new(|| char::from_u32(0xe872).unwrap());

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
