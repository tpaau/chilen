use iced::Color;

pub mod button;
pub mod scrollable;

/// Returns the arithmetic average of the two input colors.
pub fn mix_colors(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: a.r * t + b.r * (1.0 - t),
        g: a.g * t + b.g * (1.0 - t),
        b: a.b * t + b.b * (1.0 - t),
        a: a.a * t + b.a * (1.0 - t),
    }
}
