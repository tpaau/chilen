use iced::{Alignment, Color, Element};
use iced_m3::widget::{
    icon, shape,
    shapes::{self, polymorpher::RoundedPolygon},
};
use iced_widget::{column, container, stack, text};

const DEFAULT_SIZE: f32 = 128.0;
const LABEL_SIZE: f32 = font::SIZE_LARGE;
const SHAPE_LABEL_SPACE: f32 = SPACING_REGULAR;

use crate::gui::{SPACING_REGULAR, font, icons};

pub struct IconShape<'a> {
    pub shape: RoundedPolygon,
    pub shape_color: Color,
    pub icon: text::Fragment<'a>,
    pub icon_color: Color,
    pub size: f32,
    pub label: Option<(text::Fragment<'a>, Color)>,
}

impl<'a> IconShape<'a> {
    #[must_use]
    pub fn new(
        shape: RoundedPolygon,
        shape_color: Color,
        icon: text::Fragment<'a>,
        icon_color: Color,
    ) -> Self {
        Self {
            shape,
            shape_color,
            icon,
            icon_color,
            size: DEFAULT_SIZE,
            label: None,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<text::Fragment<'a>>, color: Color) -> Self {
        self.label = Some((label.into(), color));
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
        let shape = stack![
            shape(
                value.shape,
                shapes::Style {
                    fill: iced_widget::canvas::Style::Solid(value.shape_color),
                    stroke: None
                }
            )
            .size(value.size),
            container(
                icon(value.icon, value.size / 3.0)
                    .font(icons::filled())
                    .color(value.icon_color)
            )
            .center_x(value.size)
            .center_y(value.size),
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
