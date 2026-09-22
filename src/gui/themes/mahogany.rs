use iced::color;
use iced_m3::theme::Palette;

use crate::gui::themes::Template;

pub const TEMPLATE: Template = Template {
    name: "Mahogany",
    dark: Palette {
        primary: color!(0xDCBFC5),
        on_primary: color!(0x3E2B30),
        primary_container: color!(0x564146),
        on_primary_container: color!(0xF9DBE1),
        primary_fixed: color!(0xF9DBE1),
        on_primary_fixed: color!(0x27171B),
        primary_fixed_dim: color!(0xDCBFC5),
        on_primary_fixed_variant: color!(0x564146),
        inverse_primary: color!(0x6F585E),

        secondary: color!(0xD6C2C5),
        on_secondary: color!(0x3A2D30),
        secondary_container: color!(0x514346),
        on_secondary_container: color!(0xF2DDE1),
        secondary_fixed: color!(0xF2DDE1),
        on_secondary_fixed: color!(0x24191B),
        secondary_fixed_dim: color!(0xD6C2C5),
        on_secondary_fixed_variant: color!(0x514346),

        tertiary: color!(0xE3BDC5),
        on_tertiary: color!(0x422930),
        tertiary_container: color!(0x5B3F46),
        on_tertiary_container: color!(0xFFD9E1),
        tertiary_fixed: color!(0xFFD9E1),
        on_tertiary_fixed: color!(0x2B151B),
        tertiary_fixed_dim: color!(0xE3BDC5),
        on_tertiary_fixed_variant: color!(0x5B3F46),

        error: color!(0xFFB4AB),
        on_error: color!(0x690005),
        error_container: color!(0x93000A),
        on_error_container: color!(0xFFDAD6),

        surface: color!(0x151313),
        on_surface: color!(0xE8E1E1),
        surface_variant: color!(0x4A4646),
        on_surface_variant: color!(0xCCC5C5),

        surface_container_highest: color!(0x383434),
        surface_container_high: color!(0x2D292A),
        surface_container: color!(0x221F1F),
        surface_container_low: color!(0x1E1B1B),
        surface_container_lowest: color!(0x100D0E),

        inverse_surface: color!(0xE8E1E1),
        inverse_on_surface: color!(0x332F30),

        background: color!(0x151313),
        on_background: color!(0xE8E1E1),

        surface_bright: color!(0x3C3839),
        surface_dim: color!(0x151313),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x958F90),
        outline_variant: color!(0x4A4646),
    },

    light: Palette {
        primary: color!(0x6F585E),
        on_primary: color!(0xFFFFFF),
        primary_container: color!(0xF9DBE1),
        on_primary_container: color!(0x27171B),
        primary_fixed: color!(0xF9DBE1),
        on_primary_fixed: color!(0x27171B),
        primary_fixed_dim: color!(0xDCBFC5),
        on_primary_fixed_variant: color!(0x564146),
        inverse_primary: color!(0xDCBFC5),

        secondary: color!(0x6A5A5E),
        on_secondary: color!(0xFFFFFF),
        secondary_container: color!(0xF2DDE1),
        on_secondary_container: color!(0x24191B),
        secondary_fixed: color!(0xF2DDE1),
        on_secondary_fixed: color!(0x24191B),
        secondary_fixed_dim: color!(0xD6C2C5),
        on_secondary_fixed_variant: color!(0x514346),

        tertiary: color!(0x74565E),
        on_tertiary: color!(0xFFFFFF),
        tertiary_container: color!(0xFFD9E1),
        on_tertiary_container: color!(0x2B151B),
        tertiary_fixed: color!(0xFFD9E1),
        on_tertiary_fixed: color!(0x2B151B),
        tertiary_fixed_dim: color!(0xE3BDC5),
        on_tertiary_fixed_variant: color!(0x5B3F46),

        error: color!(0xBA1A1A),
        on_error: color!(0xFFFFFF),
        error_container: color!(0xFFDAD6),
        on_error_container: color!(0x410002),

        surface: color!(0xFFF8F8),
        on_surface: color!(0x1E1B1B),
        surface_variant: color!(0xE8E1E1),
        on_surface_variant: color!(0x4A4646),

        surface_container_highest: color!(0xE8E1E1),
        surface_container_high: color!(0xEEE6E7),
        surface_container: color!(0xF4ECEC),
        surface_container_low: color!(0xFAF2F2),
        surface_container_lowest: color!(0xFFFFFF),

        inverse_surface: color!(0x332F30),
        inverse_on_surface: color!(0xF7EFEF),

        background: color!(0xFFF8F8),
        on_background: color!(0x1E1B1B),

        surface_bright: color!(0xFFF8F8),
        surface_dim: color!(0xE0D8D8),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x7B7676),
        outline_variant: color!(0xCCC5C5),
    },
};
