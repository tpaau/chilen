pub mod playlist_view;
mod widgets;

use std::sync::{Arc, LazyLock, RwLock};

use iced::{
    self, Background, Color, Element, Length, Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    stream,
    widget::{column, container, row},
};
use log::error;

use crate::{gui::widgets::top_bar, music_lib::state::MusicLibrary, playback::PlaybackState};

#[derive(Debug, Clone)]
pub(super) enum Event {
    MusicLibraryChanged(Box<MusicLibrary>),
    PlaybackStateChanged(PlaybackState),
    LibraryLoadFailed(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Playlist(playlist_view::Message),
    TopBar(top_bar::Message),
}

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

pub fn send_event(event: Event) {
    match EVENT_SENDER.write().unwrap().as_mut() {
        Some(sender) => {
            if let Err(e) = sender.try_send(event) {
                error!("Could not send the event: {e}");
            }
        }
        None => {
            error!("The sender is not initialized!")
        }
    }
}

fn worker() -> impl Stream<Item = Event> {
    stream::channel(128, async |mut out| {
        let (sender, mut receiver) = mpsc::channel(128);
        *EVENT_SENDER.write().unwrap() = Some(sender);

        loop {
            let input = receiver.select_next_some().await;
            if let Err(e) = out.send(input).await {
                error!("Could not send the event, aborting: {e}");
                break;
            }
        }
    })
}

pub fn subscription() -> Subscription<Event> {
    Subscription::run(worker)
}

#[derive(Default)]
struct State {
    playlist_state: playlist_view::State,
}

fn view(state: &State) -> Element<'_, Message> {
    column([
        top_bar::view(&()).map(Message::TopBar),
        row([
            container(playlist_view::view(&state.playlist_state).map(Message::Playlist))
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
            playlist_view::update(&mut state.playlist_state, msg).map(Message::Playlist)
        }
        Message::TopBar(msg) => top_bar::update((), msg).map(Message::TopBar),
        Message::Event(event) => match event {
            Event::MusicLibraryChanged(lib) => playlist_view::update(
                &mut state.playlist_state,
                playlist_view::Message::Event(playlist_view::Event::PlaylistsChanged(
                    lib.playlists
                        .into_iter()
                        .map(|p| playlist_view::Playlist {
                            name: p.name.clone(),
                            num_tracks: p.tracks.len(),
                        })
                        .collect(),
                )),
            )
            .map(Message::Playlist),
            Event::PlaybackStateChanged(state) => todo!("Playback events"),
            Event::LibraryLoadFailed(e) => playlist_view::update(
                &mut state.playlist_state,
                playlist_view::Message::Event(playlist_view::Event::LoadFailed(e)),
            )
            .map(Message::Playlist),
        },
    }
}

pub fn start() -> iced::Result {
    iced::application(State::default, update, view)
        .subscription(|_| subscription().map(Message::Event))
        .run()
}
