pub fn cubic(t: f32, p1: f32, p2: f32) -> f32 {
    3.0 * (1.0 - t).powi(2) * t * p1 + 3.0 * (1.0 - t) * t.powi(2) * p2 + t.powi(3)
}

pub fn cubic_derivative(t: f32, p1: f32, p2: f32) -> f32 {
    3.0 * (1.0 - t).powi(2) * p1 + 6.0 * (1.0 - t) * t * (p2 - p1) + 3.0 * t.powi(2) * (1.0 - p2)
}

pub fn cubic_bezier(t: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);

    // Solve x(u) = t using Newton-Raphson iteration.
    let mut u = t;

    for _ in 0..5 {
        let x = cubic(u, x1, x2) - t;
        let derivative = cubic_derivative(u, x1, x2);

        if derivative.abs() < f32::EPSILON {
            break;
        }

        u = (u - x / derivative).clamp(0.0, 1.0);
    }

    cubic(u, y1, y2)
}

pub fn cubic_bezier_tuple(t: f32, tuple: (f32, f32, f32, f32)) -> f32 {
    cubic_bezier(t, tuple.0, tuple.1, tuple.2, tuple.3)
}

pub const EMPHASIZED_ACCELERATE: (f32, f32, f32, f32) = (0.3, 0.0, 0.8, 0.15);
pub const EMPHASIZED_DECELERATE: (f32, f32, f32, f32) = (0.05, 0.7, 0.1, 1.0);
pub const STANDARD: (f32, f32, f32, f32) = (0.2, 0.0, 0.0, 1.0);
pub const STANDARD_ACCELERATE: (f32, f32, f32, f32) = (0.3, 0.0, 1.0, 1.0);
pub const STANDARD_DECELERATE: (f32, f32, f32, f32) = (0.0, 0.0, 0.0, 1.0);
