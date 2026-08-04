mod albums;
mod artists;
mod genres;
mod tracks;

use std::sync::Arc;

use chilen_backend::music_lib::state::{Album, Artist, Track};
use iced::{Border, Color, Element, Length, Padding, Task};
use iced_m3::{HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY, theme::ColorScheme};
use iced_widget::{center, column, container, space, stack};
use log::warn;

use crate::gui::{self, BUTTON_ROUNDING, Chilen, ROUNDING_REGULAR, SPACING_SMALL, icons};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum View {
    #[default]
    Tracks,
    Albums,
    Artists,
    Genres,
}

pub struct State {
    pub view: View,
    pub tracks: Option<Vec<Option<Arc<Track>>>>,
    pub albums: Option<Vec<Option<Arc<Album>>>>,
    pub artists: Option<Vec<Option<Arc<Artist>>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeMainView(View),
    TrackButtonPoppedIn(usize),
    TrackButtonPoppedOut(usize),
    AlbumButtonPoppedIn(usize),
    AlbumButtonPoppedOut(usize),
    ArtistButtonPoppedIn(usize),
    ArtistButtonPoppedOut(usize),
}

fn button_style(
    status: iced_widget::button::Status,
    theme: &impl ColorScheme,
) -> iced_widget::button::Style {
    let content_color = theme.on_surface_variant();
    iced_widget::button::Style {
        background: Some(iced::Background::Color(match status {
            iced_widget::button::Status::Active => Color::TRANSPARENT,
            iced_widget::button::Status::Hovered => {
                content_color.scale_alpha(HOVER_STATE_LAYER_OPACITY)
            }
            iced_widget::button::Status::Pressed => {
                content_color.scale_alpha(PRESSED_STATE_LAYER_OPACITY)
            }
            iced_widget::button::Status::Disabled => {
                unreachable!("There should be no inactive buttons in the main view")
            }
        })),
        text_color: content_color,
        border: Border::default().rounded(BUTTON_ROUNDING),
        ..Default::default()
    }
}

pub fn view(state: &Chilen) -> Element<'_, gui::Message> {
    container(column![
        // TODO: Custom ordering
        {
            let index = match state.main_view.view {
                View::Tracks => 0,
                View::Albums => 1,
                View::Artists => 2,
                View::Genres => 3,
            };
            iced_m3::widget::navbar::<_, iced::Theme, iced::Renderer>(
                vec![
                    iced_m3::widget::navbar::Item {
                        icon: &icons::MUSIC_NOTE,
                        label: "Tracks",
                        message: gui::Message::MainView(Message::ChangeMainView(View::Tracks)),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::ALBUM,
                        label: "Albums",
                        message: gui::Message::MainView(Message::ChangeMainView(View::Albums)),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::ARTIST,
                        label: "Artists",
                        message: gui::Message::MainView(Message::ChangeMainView(View::Artists)),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::GENRES,
                        label: "Genres",
                        message: gui::Message::MainView(Message::ChangeMainView(View::Genres)),
                    },
                ],
                &state.theme,
            )
            .focused_index(index)
            .icon_font_active(icons::filled())
            .icon_font_inactive(icons::outlined())
        },
        {
            // FIX: This is a WORKAROUND.
            // The `Scrollable` widget doesn't correctly manage its state which makes multiple
            // scrollables in the same state tree share a state.
            //
            // There is a PR in iced that resolves this: https://github.com/iced-rs/iced/pull/3347
            let content = match state.main_view.view {
                View::Tracks => tracks::view(state),
                View::Albums => stack![albums::view(state)].into(),
                View::Artists => stack![
                    space().width(Length::Fill).height(Length::Fill),
                    artists::view(state)
                ]
                .into(),
                View::Genres => stack![
                    space().width(Length::Fill).height(Length::Fill),
                    space().width(Length::Fill).height(Length::Fill),
                    genres::view(state)
                ]
                .into(),
            };
            center(content)
        }
    ])
    .style(|_| {
        container::Style::default()
            .background(state.theme.background())
            .border(Border::default().rounded(ROUNDING_REGULAR))
    })
    .padding(Padding::new(SPACING_SMALL))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::ChangeMainView(view) => state.main_view.view = view,
        Message::TrackButtonPoppedIn(index) => {
            if let Some(lib) = &state.library {
                if index < lib.tracks.len() {
                    if let Some(tracks) = &mut state.main_view.tracks {
                        if index < tracks.len() {
                            tracks[index] = Some(lib.tracks[index].clone());
                        } else {
                            warn!(
                                "Index {index} is out of bounds for track count in the main view ({})",
                                tracks.len()
                            );
                        }
                    } else {
                        warn!("Track list in the main view state is not initialized!");
                    }
                } else {
                    warn!(
                        "Index {index} is out of bounds for track count in the music library ({})",
                        lib.tracks.len()
                    );
                }
            } else {
                warn!("Can't render track button, library not initialized!");
            }
        }
        Message::TrackButtonPoppedOut(index) => {
            if let Some(tracks) = &mut state.main_view.tracks {
                if index < tracks.len() {
                    tracks[index] = None;
                } else {
                    warn!(
                        "Index {index} is out of bounds for track count in the main view ({})",
                        tracks.len()
                    );
                }
            } else {
                warn!("Track list in the main view state is not initialized!");
            }
        }
        Message::AlbumButtonPoppedIn(index) => {
            if let Some(lib) = &state.library {
                if index < lib.albums.len() {
                    if let Some(albums) = &mut state.main_view.albums {
                        if index < albums.len() {
                            albums[index] = Some(lib.albums[index].clone());
                        } else {
                            warn!(
                                "Index {index} is out of bounds for album count in the main view ({})",
                                albums.len()
                            );
                        }
                    } else {
                        warn!("Album list in the main view state is not initialized!");
                    }
                } else {
                    warn!(
                        "Index {index} is out of bounds for album count in the music library ({})",
                        lib.albums.len()
                    );
                }
            } else {
                warn!("Can't render album button, library not initialized!");
            }
        }
        Message::AlbumButtonPoppedOut(index) => {
            if let Some(albums) = &mut state.main_view.albums {
                if index < albums.len() {
                    albums[index] = None;
                } else {
                    warn!(
                        "Index {index} is out of bounds for album count in the main view ({})",
                        albums.len()
                    );
                }
            } else {
                warn!("Album list in the main view state is not initialized!");
            }
        }
        Message::ArtistButtonPoppedIn(index) => {
            if let Some(lib) = &state.library {
                if index < lib.artists.len() {
                    if let Some(artists) = &mut state.main_view.artists {
                        if index < artists.len() {
                            artists[index] = Some(lib.artists[index].clone());
                        } else {
                            warn!(
                                "Index {index} is out of bounds for artist count in the main view ({})",
                                artists.len()
                            );
                        }
                    } else {
                        warn!("Artist list in the main view state is not initialized!");
                    }
                } else {
                    warn!(
                        "Index {index} is out of bounds for artist count in the music library ({})",
                        lib.albums.len()
                    );
                }
            } else {
                warn!("Can't render artist button, library not initialized!");
            }
        }
        Message::ArtistButtonPoppedOut(index) => {
            if let Some(artists) = &mut state.main_view.artists {
                if index < artists.len() {
                    artists[index] = None;
                } else {
                    warn!(
                        "Index {index} is out of bounds for artist count in the main view ({})",
                        artists.len()
                    );
                }
            } else {
                warn!("Artist list in the main view state is not initialized!");
            }
        }
    }
    Task::none()
}
