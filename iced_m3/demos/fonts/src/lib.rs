#[cfg(windows)]
const BYTES_REGULAR: &[u8] = include_bytes!("..\\resources\\Roboto\\NotoSans-Regular.ttf");
#[cfg(unix)]
const BYTES_REGULAR: &[u8] = include_bytes!("../resources/fonts/NotoSans-Regular.ttf");
#[cfg(windows)]
const BYTES_BOLD: &[u8] = include_bytes!("..\\resources\\Roboto\\NotoSans-Bold.ttf");
#[cfg(unix)]
const BYTES_BOLD: &[u8] = include_bytes!("../resources/fonts/NotoSans-Bold.ttf");

#[cfg(unix)]
const FILLED_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("../resources/fonts/MaterialSymbolsRounded_Filled-Regular.ttf");
#[cfg(windows)]
const FILLED_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("..\\resources\\fonts\\MaterialSymbolsRounded_Filled-Regular.ttf");

#[cfg(unix)]
const OUTLINED_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("../resources/fonts/MaterialSymbolsRounded-Regular.ttf");
#[cfg(windows)]
const OUTLINED_ICONS_FONT_BYTES: &[u8] =
    include_bytes!("..\\resources\\fonts\\MaterialSymbolsRounded-Regular.ttf");

const FILLED_ICONS_FONT_NAME: &str = "Material Symbols Rounded Filled";
const OUTLINED_ICONS_FONT_NAME: &str = "Material Symbols Rounded";

pub const ICON_SIZE_SMALLER: u32 = 16;
pub const ICON_SIZE_SMALL: u32 = 20;
pub const ICON_SIZE_REGULAR: u32 = 24;
pub const ICON_SIZE_LARGE: u32 = 28;
pub const ICON_SIZE_LARGER: u32 = 32;

pub const TEXT_SIZE_SMALLER: f32 = 12.0;
pub const TEXT_SIZE_SMALL: f32 = 14.0;
pub const TEXT_SIZE_REGULAR: f32 = 16.0;
pub const TEXT_SIZE_LARGE: f32 = 18.0;
pub const TEXT_SIZE_LARGER: f32 = 20.0;

pub const NAME: &str = "Noto Sans";

pub fn text_regular() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Normal,
        family: iced::font::Family::Name(NAME),
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

pub fn text_bold() -> iced::Font {
    iced::Font {
        weight: iced::font::Weight::Bold,
        ..text_regular()
    }
}

pub fn icons_filled() -> iced::Font {
    iced::Font {
        family: iced::font::Family::Name(FILLED_ICONS_FONT_NAME),
        weight: iced::font::Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

pub fn icons_outlined() -> iced::Font {
    iced::Font {
        family: iced::font::Family::Name(OUTLINED_ICONS_FONT_NAME),
        weight: iced::font::Weight::Normal,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

pub fn get_fonts<'a>() -> [&'a [u8]; 4] {
    [
        FILLED_ICONS_FONT_BYTES,
        OUTLINED_ICONS_FONT_BYTES,
        BYTES_REGULAR,
        BYTES_BOLD,
    ]
}
