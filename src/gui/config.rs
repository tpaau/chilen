pub struct Rounding {
    pub smaller: u32,
    pub small: u32,
    pub regular: u32,
    pub large: u32,
    pub larger: u32,
}

impl Default for Rounding {
    fn default() -> Self {
        Self {
            smaller: 12,
            small: 14,
            regular: 16,
            large: 18,
            larger: 20,
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

pub struct FontSize {
    pub smaller: u32,
    pub small: u32,
    pub regular: u32,
    pub large: u32,
    pub larger: u32,
}

impl Default for FontSize {
    fn default() -> Self {
        Self {
            smaller: 12,
            small: 14,
            regular: 16,
            large: 18,
            larger: 22,
        }
    }
}
