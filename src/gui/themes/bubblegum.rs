use iced::color;
use iced_m3::theme::Palette;

use crate::gui::themes::Template;

pub const TEMPLATE: Template = Template {
    name: "Bubblegum",
    dark: Palette {
        primary: color!(0xFFADE0),
        on_primary: color!(0x5F004D),
        primary_container: color!(0x87006D),
        on_primary_container: color!(0xFFD8ED),
        primary_fixed: color!(0xFFD8ED),
        on_primary_fixed: color!(0x3B002E),
        primary_fixed_dim: color!(0xFFADE0),
        on_primary_fixed_variant: color!(0x87006D),
        inverse_primary: color!(0xB00090),

        secondary: color!(0xEEB8CB),
        on_secondary: color!(0x492534),
        secondary_container: color!(0x623B4A),
        on_secondary_container: color!(0xFFD9E5),
        secondary_fixed: color!(0xFFD9E5),
        on_secondary_fixed: color!(0x31111F),
        secondary_fixed_dim: color!(0xEEB8CB),
        on_secondary_fixed_variant: color!(0x623B4A),

        tertiary: color!(0xFFB2BD),
        on_tertiary: color!(0x52202A),
        tertiary_container: color!(0x6E3540),
        on_tertiary_container: color!(0xFFD9DD),
        tertiary_fixed: color!(0xFFD9DD),
        on_tertiary_fixed: color!(0x380B16),
        tertiary_fixed_dim: color!(0xFFB2BD),
        on_tertiary_fixed_variant: color!(0x6E3540),

        error: color!(0xFFB4AB),
        on_error: color!(0x690005),
        error_container: color!(0x93000A),
        on_error_container: color!(0xFFDAD6),

        surface: color!(0x1B1017),
        on_surface: color!(0xF2DDE7),
        surface_variant: color!(0x51434B),
        on_surface_variant: color!(0xD5C1CB),

        surface_container_highest: color!(0x3E3138),
        surface_container_high: color!(0x33262E),
        surface_container: color!(0x281C23),
        surface_container_low: color!(0x24181F),
        surface_container_lowest: color!(0x150B11),

        inverse_surface: color!(0xF2DDE7),
        inverse_on_surface: color!(0x3A2D34),

        background: color!(0x1B1017),
        on_background: color!(0xF2DDE7),

        surface_bright: color!(0x43353D),
        surface_dim: color!(0x1B1017),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x9E8C95),
        outline_variant: color!(0x51434B),
    },

    light: Palette {
        primary: color!(0xB00090),
        on_primary: color!(0xFFFFFF),
        primary_container: color!(0xFFD8ED),
        on_primary_container: color!(0x3B002E),
        primary_fixed: color!(0xFFD8ED),
        on_primary_fixed: color!(0x3B002E),
        primary_fixed_dim: color!(0xFFADE0),
        on_primary_fixed_variant: color!(0x87006D),
        inverse_primary: color!(0xFFADE0),

        secondary: color!(0x7D5262),
        on_secondary: color!(0xFFFFFF),
        secondary_container: color!(0xFFD9E5),
        on_secondary_container: color!(0x31111F),
        secondary_fixed: color!(0xFFD9E5),
        on_secondary_fixed: color!(0x31111F),
        secondary_fixed_dim: color!(0xEEB8CB),
        on_secondary_fixed_variant: color!(0x623B4A),

        tertiary: color!(0x894C57),
        on_tertiary: color!(0xFFFFFF),
        tertiary_container: color!(0xFFD9DD),
        on_tertiary_container: color!(0x380B16),
        tertiary_fixed: color!(0xFFD9DD),
        on_tertiary_fixed: color!(0x380B16),
        tertiary_fixed_dim: color!(0xFFB2BD),
        on_tertiary_fixed_variant: color!(0x6E3540),

        error: color!(0xBA1A1A),
        on_error: color!(0xFFFFFF),
        error_container: color!(0xFFDAD6),
        on_error_container: color!(0x410002),

        surface: color!(0xFFF8F9),
        on_surface: color!(0x24181F),
        surface_variant: color!(0xF2DDE7),
        on_surface_variant: color!(0x51434B),

        surface_container_highest: color!(0xF2DDE7),
        surface_container_high: color!(0xF8E2EC),
        surface_container: color!(0xFEE8F2),
        surface_container_low: color!(0xFFF0F6),
        surface_container_lowest: color!(0xFFFFFF),

        inverse_surface: color!(0x3A2D34),
        inverse_on_surface: color!(0xFFECF4),

        background: color!(0xFFF8F9),
        on_background: color!(0x24181F),

        surface_bright: color!(0xFFF8F9),
        surface_dim: color!(0xEAD4DE),

        scrim: color!(0x000000),
        shadow: color!(0x000000),
        outline: color!(0x83727B),
        outline_variant: color!(0xD5C1CB),
    },
};
