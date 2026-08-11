#[cfg(test)]
mod tests;

use iced::{Color, color};

#[derive(Default, Clone, Copy)]
pub enum Accent {
    #[default]
    Primary,
    Secondary,
    Tertiary,
}

pub trait ColorScheme {
    fn primary(&self) -> Color;
    fn on_primary(&self) -> Color;
    fn primary_container(&self) -> Color;
    fn on_primary_container(&self) -> Color;
    fn primary_fixed(&self) -> Color;
    fn on_primary_fixed(&self) -> Color;
    fn primary_fixed_dim(&self) -> Color;
    fn on_primary_fixed_variant(&self) -> Color;
    fn inverse_primary(&self) -> Color;

    fn secondary(&self) -> Color;
    fn on_secondary(&self) -> Color;
    fn secondary_container(&self) -> Color;
    fn on_secondary_container(&self) -> Color;
    fn secondary_fixed(&self) -> Color;
    fn on_secondary_fixed(&self) -> Color;
    fn secondary_fixed_dim(&self) -> Color;
    fn on_secondary_fixed_variant(&self) -> Color;

    fn tertiary(&self) -> Color;
    fn on_tertiary(&self) -> Color;
    fn tertiary_container(&self) -> Color;
    fn on_tertiary_container(&self) -> Color;
    fn tertiary_fixed(&self) -> Color;
    fn on_tertiary_fixed(&self) -> Color;
    fn tertiary_fixed_dim(&self) -> Color;
    fn on_tertiary_fixed_variant(&self) -> Color;

    fn error(&self) -> Color;
    fn on_error(&self) -> Color;
    fn error_container(&self) -> Color;
    fn on_error_container(&self) -> Color;

    fn surface(&self) -> Color;
    fn on_surface(&self) -> Color;
    fn surface_variant(&self) -> Color;
    fn on_surface_variant(&self) -> Color;
    fn surface_container_highest(&self) -> Color;
    fn surface_container_high(&self) -> Color;
    fn surface_container(&self) -> Color;
    fn surface_container_low(&self) -> Color;
    fn surface_container_lowest(&self) -> Color;
    fn inverse_surface(&self) -> Color;
    fn inverse_on_surface(&self) -> Color;

    fn background(&self) -> Color;
    fn on_background(&self) -> Color;
    fn surface_bright(&self) -> Color;
    fn surface_dim(&self) -> Color;
    fn scrim(&self) -> Color;
    fn shadow(&self) -> Color;
    fn outline(&self) -> Color;
    fn outline_variant(&self) -> Color;
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub primary: Color,
    pub on_primary: Color,
    pub primary_container: Color,
    pub on_primary_container: Color,
    pub primary_fixed: Color,
    pub on_primary_fixed: Color,
    pub primary_fixed_dim: Color,
    pub on_primary_fixed_variant: Color,
    pub inverse_primary: Color,

    pub secondary: Color,
    pub on_secondary: Color,
    pub secondary_container: Color,
    pub on_secondary_container: Color,
    pub secondary_fixed: Color,
    pub on_secondary_fixed: Color,
    pub secondary_fixed_dim: Color,
    pub on_secondary_fixed_variant: Color,

    pub tertiary: Color,
    pub on_tertiary: Color,
    pub tertiary_container: Color,
    pub on_tertiary_container: Color,
    pub tertiary_fixed: Color,
    pub on_tertiary_fixed: Color,
    pub tertiary_fixed_dim: Color,
    pub on_tertiary_fixed_variant: Color,

    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,
    pub on_error_container: Color,

    pub surface: Color,
    pub on_surface: Color,
    pub surface_variant: Color,
    pub on_surface_variant: Color,
    pub surface_container_highest: Color,
    pub surface_container_high: Color,
    pub surface_container: Color,
    pub surface_container_low: Color,
    pub surface_container_lowest: Color,
    pub inverse_surface: Color,
    pub inverse_on_surface: Color,
    pub background: Color,
    pub on_background: Color,
    pub surface_bright: Color,
    pub surface_dim: Color,
    pub scrim: Color,
    pub shadow: Color,

    pub outline: Color,
    pub outline_variant: Color,
}

impl Palette {
    // TODO: Replace the default material color scheme with a custom one.
    pub fn default_dark() -> Self {
        Self {
            primary: color!(0xD0BCFF),
            on_primary: color!(0x381E72),
            primary_container: color!(0x4F378B),
            on_primary_container: color!(0xEADDFF),
            primary_fixed: color!(0xEADDFF),
            on_primary_fixed: color!(0x21005D),
            primary_fixed_dim: color!(0xD0BCFF),
            on_primary_fixed_variant: color!(0x4F378B),
            inverse_primary: color!(0x6750A4),

            secondary: color!(0xCCC2DC),
            on_secondary: color!(0x332D41),
            secondary_container: color!(0x4A4458),
            on_secondary_container: color!(0xE8DEF8),
            secondary_fixed: color!(0xE8DEF8),
            on_secondary_fixed: color!(0x1D192B),
            secondary_fixed_dim: color!(0xCCC2DC),
            on_secondary_fixed_variant: color!(0x4A4458),

            tertiary: color!(0xEFB8C8),
            on_tertiary: color!(0x492532),
            tertiary_container: color!(0x633B48),
            on_tertiary_container: color!(0xFFD8E4),
            tertiary_fixed: color!(0xFFD8E4),
            on_tertiary_fixed: color!(0x31111D),
            tertiary_fixed_dim: color!(0xEFB8C8),
            on_tertiary_fixed_variant: color!(0x633B48),

            error: color!(0xF2B8B5),
            on_error: color!(0x601410),
            error_container: color!(0x8C1D18),
            on_error_container: color!(0xF9DEDC),

            surface: color!(0x141218),
            on_surface: color!(0xE6E0E9),
            surface_variant: color!(0x49454F),
            on_surface_variant: color!(0xCAC4D0),
            surface_container_highest: color!(0x36343B),
            surface_container_high: color!(0x2B2930),
            surface_container: color!(0x211F26),
            surface_container_low: color!(0x1D1B20),
            surface_container_lowest: color!(0x0F0D13),
            inverse_surface: color!(0xE6E0E9),
            inverse_on_surface: color!(0x322F35),
            background: color!(0x141218),
            on_background: color!(0xE6E0E9),
            surface_bright: color!(0x3B383E),
            surface_dim: color!(0x141218),
            scrim: color!(0x000000),
            shadow: color!(0x000000),

            outline: color!(0x938F99),
            outline_variant: color!(0x49454F),
        }
    }

    pub fn default_light() -> Self {
        Self {
            primary: color!(0x6750A4),
            on_primary: color!(0xFFFFFF),
            primary_container: color!(0xEADDFF),
            on_primary_container: color!(0x4F378B),
            primary_fixed: color!(0xEADDFF),
            on_primary_fixed: color!(0x21005D),
            primary_fixed_dim: color!(0xD0BCFF),
            on_primary_fixed_variant: color!(0x4F378B),
            inverse_primary: color!(0xD0BCFF),

            secondary: color!(0x625B71),
            on_secondary: color!(0xFFFFFF),
            secondary_container: color!(0xE8DEF8),
            on_secondary_container: color!(0x4A4458),
            secondary_fixed: color!(0xE8DEF8),
            on_secondary_fixed: color!(0x1D192B),
            secondary_fixed_dim: color!(0xCCC2DC),
            on_secondary_fixed_variant: color!(0x4A4458),

            tertiary: color!(0x7D5260),
            on_tertiary: color!(0xFFFFFF),
            tertiary_container: color!(0xFFD8E4),
            on_tertiary_container: color!(0x633B48),
            tertiary_fixed: color!(0xFFD8E4),
            on_tertiary_fixed: color!(0x31111D),
            tertiary_fixed_dim: color!(0xEFB8C8),
            on_tertiary_fixed_variant: color!(0x633B48),

            error: color!(0xB3261E),
            on_error: color!(0xFFFFFF),
            error_container: color!(0xF9DEDC),
            on_error_container: color!(0x8C1D18),

            surface: color!(0xFEF7FF),
            on_surface: color!(0x1D1B20),
            surface_variant: color!(0xE7E0EC),
            on_surface_variant: color!(0x49454F),
            surface_container_highest: color!(0xE6E0E9),
            surface_container_high: color!(0xECE6F0),
            surface_container: color!(0xF3EDF7),
            surface_container_low: color!(0xF7F2FA),
            surface_container_lowest: color!(0xFFFFFF),
            inverse_surface: color!(0x322F35),
            inverse_on_surface: color!(0xF5EFF7),
            background: color!(0xFEF7FF),
            on_background: color!(0x1D1B20),
            surface_bright: color!(0xFEF7FF),
            surface_dim: color!(0xDED8E1),
            scrim: color!(0x000000),
            shadow: color!(0x000000),

            outline: color!(0x79747E),
            outline_variant: color!(0xCAC4D0),
        }
    }
}

impl ColorScheme for Palette {
    fn primary(&self) -> Color {
        self.primary
    }

    fn on_primary(&self) -> Color {
        self.on_primary
    }

    fn primary_container(&self) -> Color {
        self.primary_container
    }

    fn on_primary_container(&self) -> Color {
        self.on_primary_container
    }

    fn primary_fixed(&self) -> Color {
        self.primary_fixed
    }

    fn on_primary_fixed(&self) -> Color {
        self.on_primary_fixed
    }

    fn primary_fixed_dim(&self) -> Color {
        self.primary_fixed_dim
    }

    fn on_primary_fixed_variant(&self) -> Color {
        self.on_primary_fixed_variant
    }

    fn inverse_primary(&self) -> Color {
        self.inverse_primary
    }

    fn secondary(&self) -> Color {
        self.secondary
    }

    fn on_secondary(&self) -> Color {
        self.on_secondary
    }

    fn secondary_container(&self) -> Color {
        self.secondary_container
    }

    fn on_secondary_container(&self) -> Color {
        self.on_secondary_container
    }

    fn secondary_fixed(&self) -> Color {
        self.secondary_fixed
    }

    fn on_secondary_fixed(&self) -> Color {
        self.on_secondary_fixed
    }

    fn secondary_fixed_dim(&self) -> Color {
        self.secondary_fixed_dim
    }

    fn on_secondary_fixed_variant(&self) -> Color {
        self.on_secondary_fixed_variant
    }

    fn tertiary(&self) -> Color {
        self.tertiary
    }

    fn on_tertiary(&self) -> Color {
        self.on_tertiary
    }

    fn tertiary_container(&self) -> Color {
        self.tertiary_container
    }

    fn on_tertiary_container(&self) -> Color {
        self.on_tertiary_container
    }

    fn tertiary_fixed(&self) -> Color {
        self.tertiary_fixed
    }

    fn on_tertiary_fixed(&self) -> Color {
        self.on_tertiary_fixed
    }

    fn tertiary_fixed_dim(&self) -> Color {
        self.tertiary_fixed_dim
    }

    fn on_tertiary_fixed_variant(&self) -> Color {
        self.on_tertiary_fixed_variant
    }

    fn error(&self) -> Color {
        self.error
    }

    fn on_error(&self) -> Color {
        self.on_error
    }

    fn error_container(&self) -> Color {
        self.error_container
    }

    fn on_error_container(&self) -> Color {
        self.on_error_container
    }

    fn surface(&self) -> Color {
        self.surface
    }

    fn on_surface(&self) -> Color {
        self.on_surface
    }

    fn surface_variant(&self) -> Color {
        self.surface_variant
    }

    fn on_surface_variant(&self) -> Color {
        self.on_surface_variant
    }

    fn surface_container_highest(&self) -> Color {
        self.surface_container_highest
    }

    fn surface_container_high(&self) -> Color {
        self.surface_container_high
    }

    fn surface_container(&self) -> Color {
        self.surface_container
    }

    fn surface_container_low(&self) -> Color {
        self.surface_container_low
    }

    fn surface_container_lowest(&self) -> Color {
        self.surface_container_lowest
    }

    fn inverse_surface(&self) -> Color {
        self.inverse_surface
    }

    fn inverse_on_surface(&self) -> Color {
        self.inverse_on_surface
    }

    fn background(&self) -> Color {
        self.background
    }

    fn on_background(&self) -> Color {
        self.on_background
    }

    fn surface_bright(&self) -> Color {
        self.surface_bright
    }

    fn surface_dim(&self) -> Color {
        self.surface_dim
    }

    fn scrim(&self) -> Color {
        self.scrim
    }

    fn shadow(&self) -> Color {
        self.shadow
    }

    fn outline(&self) -> Color {
        self.outline
    }

    fn outline_variant(&self) -> Color {
        self.outline_variant
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Mode {
    #[default]
    Light,
    Dark,
}

impl Mode {
    pub fn is_dark(&self) -> bool {
        match self {
            Mode::Light => false,
            Mode::Dark => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub dark: Palette,
    pub light: Palette,
    pub mode: Mode,
}

impl Theme {
    pub fn current(&self) -> &Palette {
        match self.mode {
            Mode::Light => &self.light,
            Mode::Dark => &self.dark,
        }
    }

    pub fn default(mode: Mode) -> Self {
        Self {
            dark: Palette::default_dark(),
            light: Palette::default_light(),
            mode,
        }
    }
}

impl ColorScheme for Theme {
    fn primary(&self) -> Color {
        self.current().primary
    }

    fn on_primary(&self) -> Color {
        self.current().on_primary
    }

    fn primary_container(&self) -> Color {
        self.current().primary_container
    }

    fn on_primary_container(&self) -> Color {
        self.current().on_primary_container
    }

    fn primary_fixed(&self) -> Color {
        self.current().primary_fixed
    }

    fn on_primary_fixed(&self) -> Color {
        self.current().on_primary_fixed
    }

    fn primary_fixed_dim(&self) -> Color {
        self.current().primary_fixed_dim
    }

    fn on_primary_fixed_variant(&self) -> Color {
        self.current().on_primary_fixed_variant
    }

    fn inverse_primary(&self) -> Color {
        self.current().inverse_primary
    }

    fn secondary(&self) -> Color {
        self.current().secondary
    }

    fn on_secondary(&self) -> Color {
        self.current().on_secondary
    }

    fn secondary_container(&self) -> Color {
        self.current().secondary_container
    }

    fn on_secondary_container(&self) -> Color {
        self.current().on_secondary_container
    }

    fn secondary_fixed(&self) -> Color {
        self.current().secondary_fixed
    }

    fn on_secondary_fixed(&self) -> Color {
        self.current().on_secondary_fixed
    }

    fn secondary_fixed_dim(&self) -> Color {
        self.current().secondary_fixed_dim
    }

    fn on_secondary_fixed_variant(&self) -> Color {
        self.current().on_secondary_fixed_variant
    }

    fn tertiary(&self) -> Color {
        self.current().tertiary
    }

    fn on_tertiary(&self) -> Color {
        self.current().on_tertiary
    }

    fn tertiary_container(&self) -> Color {
        self.current().tertiary_container
    }

    fn on_tertiary_container(&self) -> Color {
        self.current().on_tertiary_container
    }

    fn tertiary_fixed(&self) -> Color {
        self.current().tertiary_fixed
    }

    fn on_tertiary_fixed(&self) -> Color {
        self.current().on_tertiary_fixed
    }

    fn tertiary_fixed_dim(&self) -> Color {
        self.current().tertiary_fixed_dim
    }

    fn on_tertiary_fixed_variant(&self) -> Color {
        self.current().on_tertiary_fixed_variant
    }

    fn error(&self) -> Color {
        self.current().error
    }

    fn on_error(&self) -> Color {
        self.current().on_error
    }

    fn error_container(&self) -> Color {
        self.current().error_container
    }

    fn on_error_container(&self) -> Color {
        self.current().on_error_container
    }

    fn surface(&self) -> Color {
        self.current().surface
    }

    fn on_surface(&self) -> Color {
        self.current().on_surface
    }

    fn surface_variant(&self) -> Color {
        self.current().surface_variant
    }

    fn on_surface_variant(&self) -> Color {
        self.current().on_surface_variant
    }

    fn surface_container_highest(&self) -> Color {
        self.current().surface_container_highest
    }

    fn surface_container_high(&self) -> Color {
        self.current().surface_container_high
    }

    fn surface_container(&self) -> Color {
        self.current().surface_container
    }

    fn surface_container_low(&self) -> Color {
        self.current().surface_container_low
    }

    fn surface_container_lowest(&self) -> Color {
        self.current().surface_container_lowest
    }

    fn inverse_surface(&self) -> Color {
        self.current().inverse_surface
    }

    fn inverse_on_surface(&self) -> Color {
        self.current().inverse_on_surface
    }

    fn background(&self) -> Color {
        self.current().background
    }

    fn on_background(&self) -> Color {
        self.current().on_background
    }

    fn surface_bright(&self) -> Color {
        self.current().surface_bright
    }

    fn surface_dim(&self) -> Color {
        self.current().surface_dim
    }

    fn scrim(&self) -> Color {
        self.current().scrim
    }

    fn shadow(&self) -> Color {
        self.current().shadow
    }

    fn outline(&self) -> Color {
        self.current().outline
    }

    fn outline_variant(&self) -> Color {
        self.current().outline_variant
    }
}
