pub mod playlist_view;
mod widgets;

use iced::{
    self, Background, Color, Element, Length, Task,
    widget::{column, container, row},
};

use crate::gui::widgets::top_bar;

#[derive(Debug, Clone)]
pub enum Message {
    Playlist(playlist_view::Message),
    TopBar(top_bar::Message),
}

#[derive(Default)]
struct State {
    playlists: Vec<playlist_view::Playlist>,
}

fn view(state: &State) -> Element<'_, Message> {
    column([
        top_bar::view(&()).map(Message::TopBar),
        row([
            container(playlist_view::view(&state.playlists).map(Message::Playlist))
                .style(|_| container::background(Background::Color(Color::from_rgb(1.0, 0.0, 0.0))))
                .width(Length::Fixed(300.0))
                .height(Length::Fill)
                .into(),
            container("Center view")
                .style(|_| container::background(Background::Color(Color::from_rgb(0.0, 1.0, 0.0))))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            container("Currently playing")
                .style(|_| container::background(Background::Color(Color::from_rgb(0.0, 0.0, 1.0))))
                .width(Length::Fixed(300.0))
                .height(Length::Fill)
                .into(),
        ])
        .into(),
    ])
    .into()
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Playlist(msg) => {
            playlist_view::update(&mut state.playlists, msg).map(Message::Playlist)
        }
        Message::TopBar(msg) => top_bar::update((), msg).map(Message::TopBar),
    }
}

pub fn start() -> iced::Result {
    iced::application(State::default, update, view)
        .subscription(|_| {
            playlist_view::subscription()
                .map(|e| Message::Playlist(playlist_view::Message::Event(e)))
        })
        .run()
}
