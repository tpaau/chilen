use std::path::PathBuf;

use iced::{Border, Color, Length, border::Radius};
use iced_widget::{Container, center, container, image, responsive, stack, text};

use crate::gui::icons;

pub fn cover_image<'a, Message, R>(
    image_path: Option<PathBuf>,
    icon: &'a char,
    icon_color: Color,
    container_color: Color,
    rounding: R,
) -> Container<'a, Message>
where
    Message: 'a,
    R: 'a + Into<Radius> + Copy,
{
    container(stack![
        responsive(move |size| center(
            text(icon)
                .font(icons::filled())
                .size(size.width.min(size.height) / 4.0)
                .color(icon_color)
        )
        .into()),
        image_path.map(|p| image(p)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .filter_method(image::FilterMethod::Linear)
            .border_radius(rounding))
    ])
    .style(move |_| {
        container::Style::default()
            .background(container_color)
            .border(Border::default().rounded(rounding))
    })
}
