use iced::{Border, Color, Element, Length};
use iced_widget::{column, container, space};

pub fn text_spacer<'a, Message: 'a>(color: Color, font_size: f32) -> Element<'a, Message> {
    column![
        space().height(Length::Fixed(font_size / 5.0)),
        container(
            space()
                .width(Length::Fixed(font_size / 3.0))
                .height(Length::Fixed(font_size / 3.0)),
        )
        .style(move |_| {
            container::Style::default()
                .background(color)
                .border(Border::default().rounded(f32::MAX))
        }),
    ]
    .into()
}
