use iced::color;
use iced_m3::theme::Palette;

use crate::gui::themes::Template;

pub const TEMPLATE: Template = Template {
    name: "Breathe",
    dark: Palette {
        primary: color!(0xBCC2FF),
        on_primary: color!(0x242B61),
        primary_container: color!(0x3B4279),
        on_primary_container: color!(0xDFE0FF),
        primary_fixed: color!(0xDFE0FF),
        on_primary_fixed: color!(0x0D154B),
        primary_fixed_dim: color!(0xBCC2FF),
        on_primary_fixed_variant: color!(0x3B4279),
        inverse_primary: color!(0x535A92),

        secondary: color!(0xC4C5DD),
        on_secondary: color!(0x2D2F42),
        secondary_container: color!(0x434559),
        on_secondary_container: color!(0xE0E0F9),
        secondary_fixed: color!(0xE0E0F9),
        on_secondary_fixed: color!(0x181A2C),
        secondary_fixed_dim: color!(0xC4C5DD),
        on_secondary_fixed_variant: color!(0x434559),

        tertiary: color!(0xE6BAD6),
        on_tertiary: color!(0x45263D),
        tertiary_container: color!(0x5D3C54),
        on_tertiary_container: color!(0xFFD7F0),
        tertiary_fixed: color!(0xFFD7F0),
        on_tertiary_fixed: color!(0x2D1127),
        tertiary_fixed_dim: color!(0xE6BAD6),
        on_tertiary_fixed_variant: color!(0x5D3C54),

        error: color!(0xFFB4AB),
        on_error: color!(0x690005),
        error_container: color!(0x93000A),
        on_error_container: color!(0xFFDAD6),

        surface: color!(0x131318),
        on_surface: color!(0xE4E1E9),
        surface_variant: color!(0x46464F),
        on_surface_variant: color!(0xC7C5D0),

        surface_container_highest: color!(0x34343A),
        surface_container_high: color!(0x29292F),
        surface_container: color!(0x1F1F25),
        surface_container_low: color!(0x1B1B21),
        surface_container_lowest: color!(0x0D0E13),

        inverse_surface: color!(0xE4E1E9),
        inverse_on_surface: color!(0x303036),

        background: color!(0x131318),
        on_background: color!(0xE4E1E9),

        surface_bright: color!(0x39393F),
        surface_dim: color!(0x131318),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x90909A),
        outline_variant: color!(0x46464F),
    },

    light: Palette {
        primary: color!(0x535A92),
        on_primary: color!(0xFFFFFF),
        primary_container: color!(0xDFE0FF),
        on_primary_container: color!(0x0D154B),
        primary_fixed: color!(0xDFE0FF),
        on_primary_fixed: color!(0x0D154B),
        primary_fixed_dim: color!(0xBCC2FF),
        on_primary_fixed_variant: color!(0x3B4279),
        inverse_primary: color!(0xBCC2FF),

        secondary: color!(0x5B5D72),
        on_secondary: color!(0xFFFFFF),
        secondary_container: color!(0xE0E0F9),
        on_secondary_container: color!(0x181A2C),
        secondary_fixed: color!(0xE0E0F9),
        on_secondary_fixed: color!(0x181A2C),
        secondary_fixed_dim: color!(0xC4C5DD),
        on_secondary_fixed_variant: color!(0x434559),

        tertiary: color!(0x77536C),
        on_tertiary: color!(0xFFFFFF),
        tertiary_container: color!(0xFFD7F0),
        on_tertiary_container: color!(0x2D1127),
        tertiary_fixed: color!(0xFFD7F0),
        on_tertiary_fixed: color!(0x2D1127),
        tertiary_fixed_dim: color!(0xE6BAD6),
        on_tertiary_fixed_variant: color!(0x5D3C54),

        error: color!(0xBA1A1A),
        on_error: color!(0xFFFFFF),
        error_container: color!(0xFFDAD6),
        on_error_container: color!(0x410002),

        surface: color!(0xFBF8FF),
        on_surface: color!(0x1B1B21),
        surface_variant: color!(0xE3E1EC),
        on_surface_variant: color!(0x46464F),

        surface_container_highest: color!(0xE4E1E9),
        surface_container_high: color!(0xE9E7EF),
        surface_container: color!(0xEFEDF4),
        surface_container_low: color!(0xF5F2FA),
        surface_container_lowest: color!(0xFFFFFF),

        inverse_surface: color!(0x303036),
        inverse_on_surface: color!(0xF2EFF7),

        background: color!(0xFBF8FF),
        on_background: color!(0x1B1B21),

        surface_bright: color!(0xFBF8FF),
        surface_dim: color!(0xDBD9E0),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x777680),
        outline_variant: color!(0xC7C5D0),
    },
};
