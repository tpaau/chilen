use iced_m3::theme::Palette;

use crate::gui::themes::Template;

pub const TEMPLATE: Template = Template {
    name: "Material",
    dark: Palette::default_dark(),
    light: Palette::default_light(),
};
