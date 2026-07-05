use iced::{
    Element, Length, Task,
    widget::{button, row, space},
    window,
};

#[derive(Debug, Copy, Clone)]
pub enum Message {
    Close,
    Minimize,
    Maximize,
}

pub fn view(_state: &()) -> Element<'_, Message> {
    row([
        space().width(Length::Fill).into(),
        button("minimize").on_press(Message::Minimize).into(),
        button("maximize").on_press(Message::Maximize).into(),
        button("quit").on_press(Message::Close).into(),
    ])
    .width(Length::Fill)
    .into()
}

pub fn update(_state: (), message: Message) -> Task<Message> {
    match message {
        Message::Minimize => window::latest().and_then(|id| {
            window::is_minimized(id).and_then(move |minimized| window::minimize(id, !minimized))
        }),
        Message::Maximize => window::latest().and_then(|id| {
            window::is_maximized(id).then(move |maximized| window::maximize(id, !maximized))
        }),
        Message::Close => window::latest().and_then(window::close),
    }
}
