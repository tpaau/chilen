use iced::color;
use iced_m3::theme::Palette;

use crate::gui::themes::Template;

pub const TEMPLATE: Template = Template {
    name: "Choco",
    dark: Palette {
        primary: color!(0xFFB3AD),
        on_primary: color!(0x571E1B),
        primary_container: color!(0x733330),
        on_primary_container: color!(0xFFDAD7),
        primary_fixed: color!(0xFFDAD7),
        on_primary_fixed: color!(0x3B0908),
        primary_fixed_dim: color!(0xFFB3AD),
        on_primary_fixed_variant: color!(0x733330),
        inverse_primary: color!(0x904A45),

        secondary: color!(0xE7BDB9),
        on_secondary: color!(0x442927),
        secondary_container: color!(0x5D3F3D),
        on_secondary_container: color!(0xFFDAD7),
        secondary_fixed: color!(0xFFDAD7),
        on_secondary_fixed: color!(0x2C1513),
        secondary_fixed_dim: color!(0xE7BDB9),
        on_secondary_fixed_variant: color!(0x5D3F3D),

        tertiary: color!(0xE1C28C),
        on_tertiary: color!(0x402D04),
        tertiary_container: color!(0x584319),
        on_tertiary_container: color!(0xFFDEA6),
        tertiary_fixed: color!(0xFFDEA6),
        on_tertiary_fixed: color!(0x271900),
        tertiary_fixed_dim: color!(0xE1C28C),
        on_tertiary_fixed_variant: color!(0x584319),

        error: color!(0xFFB4AB),
        on_error: color!(0x690005),
        error_container: color!(0x93000A),
        on_error_container: color!(0xFFDAD6),

        surface: color!(0x1A1110),
        on_surface: color!(0xF1DEDD),
        surface_variant: color!(0x534342),
        on_surface_variant: color!(0xD8C2BF),

        surface_container_highest: color!(0x3D3231),
        surface_container_high: color!(0x322827),
        surface_container: color!(0x271D1C),
        surface_container_low: color!(0x231918),
        surface_container_lowest: color!(0x140C0B),

        inverse_surface: color!(0xF1DEDD),
        inverse_on_surface: color!(0x392E2D),

        background: color!(0x1A1110),
        on_background: color!(0xF1DEDD),

        surface_bright: color!(0x423736),
        surface_dim: color!(0x1A1110),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0xA08C8A),
        outline_variant: color!(0x534342),
    },

    light: Palette {
        primary: color!(0x904A45),
        on_primary: color!(0xFFFFFF),
        primary_container: color!(0xFFDAD7),
        on_primary_container: color!(0x3B0908),
        primary_fixed: color!(0xFFDAD7),
        on_primary_fixed: color!(0x3B0908),
        primary_fixed_dim: color!(0xFFB3AD),
        on_primary_fixed_variant: color!(0x733330),
        inverse_primary: color!(0xFFB3AD),

        secondary: color!(0x775653),
        on_secondary: color!(0xFFFFFF),
        secondary_container: color!(0xFFDAD7),
        on_secondary_container: color!(0x2C1513),
        secondary_fixed: color!(0xFFDAD7),
        on_secondary_fixed: color!(0x2C1513),
        secondary_fixed_dim: color!(0xE7BDB9),
        on_secondary_fixed_variant: color!(0x5D3F3D),

        tertiary: color!(0x725B2E),
        on_tertiary: color!(0xFFFFFF),
        tertiary_container: color!(0xFFDEA6),
        on_tertiary_container: color!(0x271900),
        tertiary_fixed: color!(0xFFDEA6),
        on_tertiary_fixed: color!(0x271900),
        tertiary_fixed_dim: color!(0xE1C28C),
        on_tertiary_fixed_variant: color!(0x584319),

        error: color!(0xBA1A1A),
        on_error: color!(0xFFFFFF),
        error_container: color!(0xFFDAD6),
        on_error_container: color!(0x410002),

        surface: color!(0xFFF8F7),
        on_surface: color!(0x231918),
        surface_variant: color!(0xF5DDDB),
        on_surface_variant: color!(0x534342),

        surface_container_highest: color!(0xF1DEDD),
        surface_container_high: color!(0xF6E4E2),
        surface_container: color!(0xFCEAE8),
        surface_container_low: color!(0xFFF0EF),
        surface_container_lowest: color!(0xFFFFFF),

        inverse_surface: color!(0x392E2D),
        inverse_on_surface: color!(0xFFEDEB),

        background: color!(0xFFF8F7),
        on_background: color!(0x231918),

        surface_bright: color!(0xFFF8F7),
        surface_dim: color!(0xE8D6D4),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x857371),
        outline_variant: color!(0xD8C2BF),
    },
};
