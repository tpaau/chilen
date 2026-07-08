pub mod styles;

use std::str::FromStr;

use iced::Color;

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
            primary: Color::from_str("#D0BCFF").unwrap(),
            on_primary: Color::from_str("#381E72").unwrap(),
            primary_container: Color::from_str("#4F378B").unwrap(),
            on_primary_container: Color::from_str("#EADDFF").unwrap(),
            primary_fixed: Color::from_str("#EADDFF").unwrap(),
            on_primary_fixed: Color::from_str("#21005D").unwrap(),
            primary_fixed_dim: Color::from_str("#D0BCFF").unwrap(),
            on_primary_fixed_variant: Color::from_str("#4F378B").unwrap(),
            inverse_primary: Color::from_str("#6750A4").unwrap(),

            secondary: Color::from_str("#CCC2DC").unwrap(),
            on_secondary: Color::from_str("#332D41").unwrap(),
            secondary_container: Color::from_str("#4A4458").unwrap(),
            on_secondary_container: Color::from_str("#E8DEF8").unwrap(),
            secondary_fixed: Color::from_str("#E8DEF8").unwrap(),
            on_secondary_fixed: Color::from_str("#1D192B").unwrap(),
            secondary_fixed_dim: Color::from_str("#CCC2DC").unwrap(),
            on_secondary_fixed_variant: Color::from_str("#4A4458").unwrap(),

            tertiary: Color::from_str("#EFB8C8").unwrap(),
            on_tertiary: Color::from_str("#492532").unwrap(),
            tertiary_container: Color::from_str("#633B48").unwrap(),
            on_tertiary_container: Color::from_str("#FFD8E4").unwrap(),
            tertiary_fixed: Color::from_str("#FFD8E4").unwrap(),
            on_tertiary_fixed: Color::from_str("#31111D").unwrap(),
            tertiary_fixed_dim: Color::from_str("#EFB8C8").unwrap(),
            on_tertiary_fixed_variant: Color::from_str("#633B48").unwrap(),

            error: Color::from_str("#F2B8B5").unwrap(),
            on_error: Color::from_str("#601410").unwrap(),
            error_container: Color::from_str("#8C1D18").unwrap(),
            on_error_container: Color::from_str("#F9DEDC").unwrap(),

            surface: Color::from_str("#141218").unwrap(),
            on_surface: Color::from_str("#E6E0E9").unwrap(),
            surface_variant: Color::from_str("#49454F").unwrap(),
            on_surface_variant: Color::from_str("#CAC4D0").unwrap(),
            surface_container_highest: Color::from_str("#36343B").unwrap(),
            surface_container_high: Color::from_str("#2B2930").unwrap(),
            surface_container: Color::from_str("#211F26").unwrap(),
            surface_container_low: Color::from_str("#1D1B20").unwrap(),
            surface_container_lowest: Color::from_str("#0F0D13").unwrap(),
            inverse_surface: Color::from_str("#E6E0E9").unwrap(),
            inverse_on_surface: Color::from_str("#322F35").unwrap(),
            background: Color::from_str("#141218").unwrap(),
            on_background: Color::from_str("#E6E0E9").unwrap(),
            surface_bright: Color::from_str("#3B383E").unwrap(),
            surface_dim: Color::from_str("#141218").unwrap(),
            scrim: Color::from_str("#000000").unwrap(),
            shadow: Color::from_str("#000000").unwrap(),

            outline: Color::from_str("#938F99").unwrap(),
            outline_variant: Color::from_str("#49454F").unwrap(),
        }
    }

    pub fn default_light() -> Self {
        Self {
            primary: Color::from_str("#6750A4").unwrap(),
            on_primary: Color::from_str("#FFFFFF").unwrap(),
            primary_container: Color::from_str("#EADDFF").unwrap(),
            on_primary_container: Color::from_str("#4F378B").unwrap(),
            primary_fixed: Color::from_str("#EADDFF").unwrap(),
            on_primary_fixed: Color::from_str("#21005D").unwrap(),
            primary_fixed_dim: Color::from_str("#D0BCFF").unwrap(),
            on_primary_fixed_variant: Color::from_str("#4F378B").unwrap(),
            inverse_primary: Color::from_str("#D0BCFF").unwrap(),

            secondary: Color::from_str("#625B71").unwrap(),
            on_secondary: Color::from_str("#FFFFFF").unwrap(),
            secondary_container: Color::from_str("#E8DEF8").unwrap(),
            on_secondary_container: Color::from_str("#4A4458").unwrap(),
            secondary_fixed: Color::from_str("#E8DEF8").unwrap(),
            on_secondary_fixed: Color::from_str("#1D192B").unwrap(),
            secondary_fixed_dim: Color::from_str("#CCC2DC").unwrap(),
            on_secondary_fixed_variant: Color::from_str("#4A4458").unwrap(),

            tertiary: Color::from_str("#7D5260").unwrap(),
            on_tertiary: Color::from_str("#FFFFFF").unwrap(),
            tertiary_container: Color::from_str("#FFD8E4").unwrap(),
            on_tertiary_container: Color::from_str("#633B48").unwrap(),
            tertiary_fixed: Color::from_str("#FFD8E4").unwrap(),
            on_tertiary_fixed: Color::from_str("#31111D").unwrap(),
            tertiary_fixed_dim: Color::from_str("#EFB8C8").unwrap(),
            on_tertiary_fixed_variant: Color::from_str("#633B48").unwrap(),

            error: Color::from_str("#B3261E").unwrap(),
            on_error: Color::from_str("#FFFFFF").unwrap(),
            error_container: Color::from_str("#F9DEDC").unwrap(),
            on_error_container: Color::from_str("#8C1D18").unwrap(),

            surface: Color::from_str("#FEF7FF").unwrap(),
            on_surface: Color::from_str("#1D1B20").unwrap(),
            surface_variant: Color::from_str("#E7E0EC").unwrap(),
            on_surface_variant: Color::from_str("#49454F").unwrap(),
            surface_container_highest: Color::from_str("#E6E0E9").unwrap(),
            surface_container_high: Color::from_str("#ECE6F0").unwrap(),
            surface_container: Color::from_str("#F3EDF7").unwrap(),
            surface_container_low: Color::from_str("#F7F2FA").unwrap(),
            surface_container_lowest: Color::from_str("#FFFFFF").unwrap(),
            inverse_surface: Color::from_str("#322F35").unwrap(),
            inverse_on_surface: Color::from_str("#F5EFF7").unwrap(),
            background: Color::from_str("#FEF7FF").unwrap(),
            on_background: Color::from_str("#1D1B20").unwrap(),
            surface_bright: Color::from_str("#FEF7FF").unwrap(),
            surface_dim: Color::from_str("#DED8E1").unwrap(),
            scrim: Color::from_str("#000000").unwrap(),
            shadow: Color::from_str("#000000").unwrap(),

            outline: Color::from_str("#79747E").unwrap(),
            outline_variant: Color::from_str("#CAC4D0").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    dark: Palette,
    light: Palette,
    dark_mode: bool,
}

impl Theme {
    pub fn current(&self) -> &Palette {
        match self.dark_mode {
            true => &self.dark,
            false => &self.light,
        }
    }

    pub fn default(dark_mode: bool) -> Self {
        // TODO: Add a default light theme
        Self {
            dark: Palette::default_dark(),
            light: Palette::default_light(),
            dark_mode,
        }
    }
}
