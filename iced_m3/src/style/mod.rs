mod scrollable;
mod shadow;

pub use scrollable::*;
pub use shadow::*;

use iced::Color;

/// Returns the arithmetic average of the two input colors.
pub fn mix_colors(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r * (1.0 - t) + b.r * t,
        g: a.g * (1.0 - t) + b.g * t,
        b: a.b * (1.0 - t) + b.b * t,
        a: a.a * (1.0 - t) + b.a * t,
    }
}
