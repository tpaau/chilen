use iced::border::Radius;

pub struct Rounding {
    pub smaller: Radius,
    pub small: Radius,
    pub regular: Radius,
    pub large: Radius,
    pub larger: Radius,
}

impl Default for Rounding {
    fn default() -> Self {
        Self {
            smaller: 12.into(),
            small: 14.into(),
            regular: 16.into(),
            large: 18.into(),
            larger: 20.into(),
        }
    }
}

pub struct Spacing {
    pub smaller: u32,
    pub small: u32,
    pub regular: u32,
    pub large: u32,
    pub larger: u32,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            smaller: 8,
            small: 12,
            regular: 16,
            large: 20,
            larger: 24,
        }
    }
}
