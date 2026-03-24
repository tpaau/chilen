use iced::{
    self, Element,
    widget::{button, text},
};

#[derive(Debug, Copy, Clone)]
enum Message {
    Test,
}

fn view(something: &bool) -> Element<'_, Message> {
    button(text(something)).on_press(Message::Test).into()
}

fn update(something: &mut bool, message: Message) {
    match message {
        Message::Test => *something = !*something,
    }
}

pub fn start() -> iced::Result {
    iced::run(update, view)
}
