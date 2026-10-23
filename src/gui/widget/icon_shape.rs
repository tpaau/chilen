use iced::{Alignment, Color, Element};
use iced_core::text::IntoFragment;
use iced_m3::{
    animation::motion::Spring,
    widget::{
        icon, shape,
        shapes::{self, polymorpher::RoundedPolygon},
    },
};
use iced_widget::{column, container, stack, text};

const DEFAULT_SIZE: f32 = 128.0;
const LABEL_SIZE: f32 = font::SIZE_LARGE;
const SHAPE_LABEL_SPACE: f32 = SPACING_REGULAR;

use crate::gui::{SPACING_REGULAR, font, icons};

pub struct IconShape<'a> {
    shape: RoundedPolygon,
    shape_color: Color,
    icon: text::Fragment<'a>,
    icon_color: Color,
    size: f32,
    icon_size: Option<f32>,
    label: Option<(text::Fragment<'a>, Color)>,
    transition_spring: Option<Spring>,
}

impl<'a> IconShape<'a> {
    #[must_use]
    pub fn new(
        shape: RoundedPolygon,
        shape_color: Color,
        icon: impl IntoFragment<'a>,
        icon_color: Color,
    ) -> Self {
        Self {
            shape,
            shape_color,
            icon: icon.into_fragment(),
            icon_color,
            size: DEFAULT_SIZE,
            label: None,
            icon_size: None,
            transition_spring: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<text::Fragment<'a>>, color: Color) -> Self {
        self.label = Some((label.into(), color));
        self
    }

    #[must_use]
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = Some(size);
        self
    }

    #[must_use]
    pub fn transition_spring(mut self, spring: Spring) -> Self {
        self.transition_spring = Some(spring);
        self
    }
}

impl<'a, Message, Theme, Renderer> From<IconShape<'a>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + iced_widget::container::Catalog + iced_widget::text::Catalog,
    Renderer: 'a + iced_widget::renderer::geometry::Renderer + iced_core::text::Renderer,
    Renderer::Font: From<iced::Font>,
    Theme: iced_widget::text::Catalog<Class<'a>: From<iced_widget::text::StyleFn<'a, Theme>>>,
{
    fn from(value: IconShape<'a>) -> Self {
        let icon = icon(value.icon, value.icon_size.unwrap_or(value.size / 3.0))
            .font(icons::filled())
            .color(value.icon_color);
        let icon = if let Some(size) = value.icon_size {
            icon.size(size)
        } else {
            icon
        };

        let shape = stack![
            shape(
                value.shape,
                shapes::Style {
                    fill: iced_widget::canvas::Style::Solid(value.shape_color),
                    stroke: None
                }
            )
            .transition_spring_maybe(value.transition_spring)
            .size(value.size),
            container(icon).center_x(value.size).center_y(value.size),
        ];

        match value.label {
            Some((label, label_color)) => {
                column![shape, text(label).size(LABEL_SIZE).color(label_color)]
                    .spacing(SHAPE_LABEL_SPACE)
                    .align_x(Alignment::Center)
                    .into()
            }
            None => shape.into(),
        }
    }
}
