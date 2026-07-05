use std::sync::{Arc, LazyLock, RwLock};

use iced::{
    Element, Subscription, Task,
    futures::{SinkExt, Stream, StreamExt, channel::mpsc},
    stream,
    widget::{button, column, text},
};
use log::error;

use crate::music_lib::{create_playlist, state::get_playlists};

#[derive(Debug, Clone)]
pub enum Event {
    PlaylistsChanged(Vec<Playlist>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Create,
    Event(Event),
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub name: String,
    pub num_tracks: usize,
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

fn playlist_worker() -> impl Stream<Item = Event> {
    stream::channel(128, async |mut out| {
        let (sender, mut receiver) = mpsc::channel(128);
        *EVENT_SENDER.write().unwrap() = Some(sender);

        let playlists = match get_playlists() {
            Ok(p) => p
                .into_iter()
                .map(|p| Playlist {
                    name: p.name.clone(),
                    num_tracks: p.tracks.len(),
                })
                .collect::<Vec<_>>(),
            Err(e) => {
                error!("Couldn't get the playlists from the library: {e}");
                return;
            }
        };

        if let Err(e) = out.send(Event::PlaylistsChanged(playlists)).await {
            error!("Could not send the event, aborting: {e}");
            return;
        }

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
    Subscription::run(playlist_worker)
}

pub fn view(state: &[Playlist]) -> Element<'_, Message> {
    column![
        column(
            state
                .iter()
                .map(|p| text!("Playlist \"{}\", tracks: {}", p.name, p.num_tracks).into())
        )
        .padding(12),
        button("Hello!").on_press(Message::Create)
    ]
    .into()
}

pub fn update(state: &mut Vec<Playlist>, message: Message) -> Task<Message> {
    match message {
        Message::Create => {
            if let Err(e) = create_playlist(format!("Hello {}", state.len()), &None) {
                error!(
                    "Could not create a playlist, this shouldn't happen in the finished app: {e}"
                );
            }
            Task::none()
        }
        Message::Event(event) => match event {
            Event::PlaylistsChanged(playlists) => {
                *state = playlists;
                Task::none()
            }
        },
    }
}
