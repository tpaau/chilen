mod widgets;

use iced::{
    self, Element, Length,
    widget::{button, column, container, row, text},
};

#[derive(Debug, Copy, Clone)]
enum Message {
    Test,
}

fn view(something: &bool) -> Element<'_, Message> {
    row([
        container(column((0..5).map(|i| text!("Playlist {i}").into())).padding(12))
            .width(Length::Fixed(300.0))
            .height(Length::Fill)
            .into(),
        container("Center view")
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        container("Playing")
            .width(Length::Fixed(300.0))
            .height(Length::Fill)
            .into(),
    ])
    .into()
    // column([
    //     button(text(something)).on_press(Message::Test).into(),
    //     button(text(something)).on_press(Message::Test).into(),
    // ])
    // .into()
}

fn update(something: &mut bool, message: Message) {
    match message {
        Message::Test => *something = !*something,
    }
}

pub fn start() -> iced::Result {
    iced::run(update, view)
}
