use iced::color;
use iced_m3::theme::Palette;

use crate::gui::themes::Template;

pub const TEMPLATE: Template = Template {
    name: "Watcher",
    dark: Palette {
        primary: color!(0xC0C6D5),
        on_primary: color!(0x2A313C),
        primary_container: color!(0x404753),
        on_primary_container: color!(0xDCE2F2),
        primary_fixed: color!(0xDCE2F2),
        on_primary_fixed: color!(0x151C27),
        primary_fixed_dim: color!(0xC0C6D5),
        on_primary_fixed_variant: color!(0x404753),
        inverse_primary: color!(0x585F6B),

        secondary: color!(0xC4C6CF),
        on_secondary: color!(0x2D3038),
        secondary_container: color!(0x43474E),
        on_secondary_container: color!(0xE0E2EC),
        secondary_fixed: color!(0xE0E2EC),
        on_secondary_fixed: color!(0x181C22),
        secondary_fixed_dim: color!(0xC4C6CF),
        on_secondary_fixed_variant: color!(0x43474E),

        tertiary: color!(0xBDC7DC),
        on_tertiary: color!(0x273141),
        tertiary_container: color!(0x3D4758),
        on_tertiary_container: color!(0xD9E3F8),
        tertiary_fixed: color!(0xD9E3F8),
        on_tertiary_fixed: color!(0x121C2B),
        tertiary_fixed_dim: color!(0xBDC7DC),
        on_tertiary_fixed_variant: color!(0x3D4758),

        error: color!(0xFFB4AB),
        on_error: color!(0x690005),
        error_container: color!(0x93000A),
        on_error_container: color!(0xFFDAD6),

        surface: color!(0x131315),
        on_surface: color!(0xE4E2E3),
        surface_variant: color!(0x474648),
        on_surface_variant: color!(0xC8C6C7),

        surface_container_highest: color!(0x353536),
        surface_container_high: color!(0x2A2A2B),
        surface_container: color!(0x1F1F21),
        surface_container_low: color!(0x1B1B1D),
        surface_container_lowest: color!(0x0E0E0F),

        inverse_surface: color!(0xE4E2E3),
        inverse_on_surface: color!(0x303032),

        background: color!(0x131315),
        on_background: color!(0xE4E2E3),

        surface_bright: color!(0x39393A),
        surface_dim: color!(0x131315),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x929092),
        outline_variant: color!(0x474648),
    },

    light: Palette {
        primary: color!(0x585F6B),
        on_primary: color!(0xFFFFFF),
        primary_container: color!(0xDCE2F2),
        on_primary_container: color!(0x151C27),
        primary_fixed: color!(0xDCE2F2),
        on_primary_fixed: color!(0x151C27),
        primary_fixed_dim: color!(0xC0C6D5),
        on_primary_fixed_variant: color!(0x404753),
        inverse_primary: color!(0xC0C6D5),

        secondary: color!(0x5B5E66),
        on_secondary: color!(0xFFFFFF),
        secondary_container: color!(0xE0E2EC),
        on_secondary_container: color!(0x181C22),
        secondary_fixed: color!(0xE0E2EC),
        on_secondary_fixed: color!(0x181C22),
        secondary_fixed_dim: color!(0xC4C6CF),
        on_secondary_fixed_variant: color!(0x43474E),

        tertiary: color!(0x555F71),
        on_tertiary: color!(0xFFFFFF),
        tertiary_container: color!(0xD9E3F8),
        on_tertiary_container: color!(0x121C2B),
        tertiary_fixed: color!(0xD9E3F8),
        on_tertiary_fixed: color!(0x121C2B),
        tertiary_fixed_dim: color!(0xBDC7DC),
        on_tertiary_fixed_variant: color!(0x3D4758),

        error: color!(0xBA1A1A),
        on_error: color!(0xFFFFFF),
        error_container: color!(0xFFDAD6),
        on_error_container: color!(0x410002),

        surface: color!(0xFBF8FA),
        on_surface: color!(0x1B1B1D),
        surface_variant: color!(0xE4E2E3),
        on_surface_variant: color!(0x474648),

        surface_container_highest: color!(0xE4E2E3),
        surface_container_high: color!(0xEAE7E9),
        surface_container: color!(0xF0EDEE),
        surface_container_low: color!(0xF6F3F4),
        surface_container_lowest: color!(0xFFFFFF),

        inverse_surface: color!(0x303032),
        inverse_on_surface: color!(0xF3F0F1),

        background: color!(0xFBF8FA),
        on_background: color!(0x1B1B1D),

        surface_bright: color!(0xFBF8FA),
        surface_dim: color!(0xDCD9DB),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x787778),
        outline_variant: color!(0xC8C6C7),
    },
};
