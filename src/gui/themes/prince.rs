use iced::color;
use iced_m3::theme::Palette;

use crate::gui::themes::Template;

pub const TEMPLATE: Template = Template {
    name: "Prince",
    dark: Palette {
        primary: color!(0xFFB688),
        on_primary: color!(0x512400),
        primary_container: color!(0x6E380F),
        on_primary_container: color!(0xFFDBC7),
        primary_fixed: color!(0xFFDBC7),
        on_primary_fixed: color!(0x311300),
        primary_fixed_dim: color!(0xFFB688),
        on_primary_fixed_variant: color!(0x6E380F),
        inverse_primary: color!(0x8B4F24),

        secondary: color!(0xE5BFA9),
        on_secondary: color!(0x432B1C),
        secondary_container: color!(0x5B4130),
        on_secondary_container: color!(0xFFDBC7),
        secondary_fixed: color!(0xFFDBC7),
        on_secondary_fixed: color!(0x2B1709),
        secondary_fixed_dim: color!(0xE5BFA9),
        on_secondary_fixed_variant: color!(0x5B4130),

        tertiary: color!(0xCACA93),
        on_tertiary: color!(0x32320A),
        tertiary_container: color!(0x48491E),
        on_tertiary_container: color!(0xE7E6AD),
        tertiary_fixed: color!(0xE7E6AD),
        on_tertiary_fixed: color!(0x1C1D00),
        tertiary_fixed_dim: color!(0xCACA93),
        on_tertiary_fixed_variant: color!(0x48491E),

        error: color!(0xFFB4AB),
        on_error: color!(0x690005),
        error_container: color!(0x93000A),
        on_error_container: color!(0xFFDAD6),

        surface: color!(0x19120D),
        on_surface: color!(0xF0DFD7),
        surface_variant: color!(0x52443C),
        on_surface_variant: color!(0xD7C3B8),

        surface_container_highest: color!(0x3D332D),
        surface_container_high: color!(0x312823),
        surface_container: color!(0x261E19),
        surface_container_low: color!(0x221A15),
        surface_container_lowest: color!(0x140D08),

        inverse_surface: color!(0xF0DFD7),
        inverse_on_surface: color!(0x382E29),

        background: color!(0x19120D),
        on_background: color!(0xF0DFD7),

        surface_bright: color!(0x413731),
        surface_dim: color!(0x19120D),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x9F8D83),
        outline_variant: color!(0x52443C),
    },

    light: Palette {
        primary: color!(0x8B4F24),
        on_primary: color!(0xFFFFFF),
        primary_container: color!(0xFFDBC7),
        on_primary_container: color!(0x311300),
        primary_fixed: color!(0xFFDBC7),
        on_primary_fixed: color!(0x311300),
        primary_fixed_dim: color!(0xFFB688),
        on_primary_fixed_variant: color!(0x6E380F),
        inverse_primary: color!(0xFFB688),

        secondary: color!(0x755846),
        on_secondary: color!(0xFFFFFF),
        secondary_container: color!(0xFFDBC7),
        on_secondary_container: color!(0x2B1709),
        secondary_fixed: color!(0xFFDBC7),
        on_secondary_fixed: color!(0x2B1709),
        secondary_fixed_dim: color!(0xE5BFA9),
        on_secondary_fixed_variant: color!(0x5B4130),

        tertiary: color!(0x606134),
        on_tertiary: color!(0xFFFFFF),
        tertiary_container: color!(0xE7E6AD),
        on_tertiary_container: color!(0x1C1D00),
        tertiary_fixed: color!(0xE7E6AD),
        on_tertiary_fixed: color!(0x1C1D00),
        tertiary_fixed_dim: color!(0xCACA93),
        on_tertiary_fixed_variant: color!(0x48491E),

        error: color!(0xBA1A1A),
        on_error: color!(0xFFFFFF),
        error_container: color!(0xFFDAD6),
        on_error_container: color!(0x410002),

        surface: color!(0xFFF8F5),
        on_surface: color!(0x221A15),
        surface_variant: color!(0xF4DED3),
        on_surface_variant: color!(0x52443C),

        surface_container_highest: color!(0xF0DFD7),
        surface_container_high: color!(0xF6E5DC),
        surface_container: color!(0xFCEBE2),
        surface_container_low: color!(0xFFF1EA),
        surface_container_lowest: color!(0xFFFFFF),

        inverse_surface: color!(0x382E29),
        inverse_on_surface: color!(0xFFEDE5),

        background: color!(0xFFF8F5),
        on_background: color!(0x221A15),

        surface_bright: color!(0xFFF8F5),
        surface_dim: color!(0xE7D7CE),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x84746A),
        outline_variant: color!(0xD7C3B8),
    },
};
