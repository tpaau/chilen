use iced::{Color, Shadow, Vector};

#[derive(Default, Clone, Copy)]
pub struct Elevation {
    value: f32,
}

impl From<f32> for Elevation {
    fn from(val: f32) -> Self {
        Elevation {
            value: val.clamp(0.0, 1.0),
        }
    }
}

impl Elevation {
    /// 0 -> No elevation
    /// 1 -> Max elevation
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }

    fn alpha(&self) -> f32 {
        self.value / 2.0
    }

    fn offset(&self) -> Vector {
        Vector::new(0.0, self.value * 8.0)
    }

    fn blur_radius(&self) -> f32 {
        self.value * 18.0
    }
}

pub fn shadow<E: Into<Elevation>>(color: Color, elevation: E) -> Shadow {
    let elevation = elevation.into();
    Shadow {
        color: color.scale_alpha(elevation.alpha()),
        offset: elevation.offset(),
        blur_radius: elevation.blur_radius(),
    }
}
