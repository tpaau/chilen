use std::path::PathBuf;

use iced::{Border, Color, Element, Length, border::Radius};
use iced_widget::{center, container, image, stack, text};

use crate::gui::icons;

pub struct CoverImage {
    pub image_path: Option<PathBuf>,
    pub icon: char,
    pub icon_size: f32,
    pub icon_color: Color,
    pub container_color: Color,
    pub radius: Radius,
    pub opacity: f32,
    pub width: Length,
    pub height: Length,
}

impl<'a, Message> From<CoverImage> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(value: CoverImage) -> Self {
        let path_is_none = value.image_path.is_none();
        let cover = value.image_path.map(|p| {
            image(p)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Cover)
                .filter_method(image::FilterMethod::Linear)
                .border_radius(value.radius)
                .opacity(value.opacity)
        });

        // Not ideal since it won't display an icon for non-opaque covers if the image exists but can't be loaded
        if value.opacity == 1.0 || path_is_none {
            container(stack![
                center(
                    text(value.icon)
                        .font(icons::filled())
                        .size(value.icon_size)
                        .color(value.icon_color)
                ),
                cover
            ])
            .style(move |_| {
                container::Style::default()
                    .background(value.container_color)
                    .border(Border::default().rounded(value.radius))
            })
            .width(value.width)
            .height(value.height)
            .into()
        } else {
            container(cover)
                .width(value.width)
                .height(value.height)
                .into()
        }
    }
}
