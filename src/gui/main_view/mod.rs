mod albums;
mod artists;
mod genres;
mod tracks;

use chilen_backend::music_lib::state::MusicLibrary;
use iced::{Border, Color, Element, Length, Padding, Task};
use iced_m3::{HOVER_STATE_LAYER_OPACITY, PRESSED_STATE_LAYER_OPACITY, theme::ColorScheme};
use iced_widget::{center, column, container, space, stack, text};

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
    /// Holds which elements are visible and which are not.
    /// Sadly I couldn't use a range for this, it would've used much less memory but it lagged too
    /// much when I was scrolling erratically.
    pub visible: Option<Vec<bool>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    ChangeView(View),
    ButtonPoppedIn(usize),
    ButtonPoppedOut(usize),
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
                        message: gui::Message::MainView(Message::ChangeView(View::Tracks)),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::ALBUM,
                        label: "Albums",
                        message: gui::Message::MainView(Message::ChangeView(View::Albums)),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::ARTIST,
                        label: "Artists",
                        message: gui::Message::MainView(Message::ChangeView(View::Artists)),
                    },
                    iced_m3::widget::navbar::Item {
                        icon: &icons::GENRES,
                        label: "Genres",
                        message: gui::Message::MainView(Message::ChangeView(View::Genres)),
                    },
                ],
                &state.theme,
            )
            .focused_index(index)
            .icon_font_active(icons::filled())
            .icon_font_inactive(icons::outlined())
        },
        {
            if let Some(lib) = &state.library {
                // FIX: This is a WORKAROUND.
                // The `Scrollable` widget doesn't correctly manage its state which makes multiple
                // scrollables in the same state tree share a state.
                //
                // There is a PR in iced that resolves this: https://github.com/iced-rs/iced/pull/3347
                let content = match state.main_view.view {
                    View::Tracks => tracks::view(state, lib),
                    View::Albums => stack![albums::view(state, lib)].into(),
                    View::Artists => stack![
                        space().width(Length::Fill).height(Length::Fill),
                        artists::view(state, lib)
                    ]
                    .into(),
                    View::Genres => stack![
                        space().width(Length::Fill).height(Length::Fill),
                        space().width(Length::Fill).height(Length::Fill),
                        genres::view(state, lib)
                    ]
                    .into(),
                };
                center(content)
            } else {
                center(text("Loading..."))
            }
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

fn init_visible(lib: &MusicLibrary, view: &View) -> Vec<bool> {
    match view {
        View::Tracks => vec![false; lib.tracks.len()],
        View::Albums => vec![false; lib.albums.len()],
        View::Artists => vec![false; lib.artists.len()],
        View::Genres => vec![false; lib.genres.len()],
    }
}

fn change_button(vec: &mut [bool], index: usize, visible: bool) {
    if index < vec.len() {
        vec[index] = visible;
    }
}

pub fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    if state.main_view.visible.is_none()
        && let Some(lib) = &state.library
        && !matches!(message, Message::ChangeView(_))
    {
        state.main_view.visible = Some(init_visible(lib, &state.main_view.view));
    }
    match message {
        Message::ChangeView(view) => {
            if state.main_view.view != view {
                state.main_view.visible =
                    state.library.as_ref().map(|lib| init_visible(lib, &view));
                state.main_view.view = view
            }
        }
        Message::ButtonPoppedIn(index) => {
            change_button(state.main_view.visible.as_mut().unwrap(), index, true);
        }
        Message::ButtonPoppedOut(index) => {
            change_button(state.main_view.visible.as_mut().unwrap(), index, false);
        }
    }
    Task::none()
}
