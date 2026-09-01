use std::path::PathBuf;

use iced::{Border, Color, Length, border::Radius};
use iced_widget::{Container, center, container, image, stack, text};

use crate::gui::icons;

pub fn cover_image<'a, Message, R>(
    image_path: Option<PathBuf>,
    icon: &'a char,
    icon_size: f32,
    icon_color: Color,
    container_color: Color,
    rounding: R,
    opacity: f32,
) -> Container<'a, Message>
where
    Message: 'a,
    R: 'a + Into<Radius> + Copy,
{
    let path_is_none = image_path.is_none();
    let cover = image_path.map(|p| {
        image(p)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .filter_method(image::FilterMethod::Linear)
            .border_radius(rounding)
            .opacity(opacity)
    });

    // Not ideal since it won't display an icon for non-opaque covers if the image exists but can't be loaded
    if opacity == 1.0 || path_is_none {
        container(stack![
            center(
                text(icon)
                    .font(icons::filled())
                    .size(icon_size)
                    .color(icon_color)
            ),
            cover
        ])
        .style(move |_| {
            container::Style::default()
                .background(container_color)
                .border(Border::default().rounded(rounding))
        })
    } else {
        container(cover)
    }
}
