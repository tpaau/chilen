use std::time::Duration;

use chilen_backend::{
    music_lib::{LibraryItem, is_favorite},
    playback::LoopState,
};
use iced::{Alignment, Element, Length};
use iced_core::text::IntoFragment;
use iced_m3::{
    theme::{Accent, ColorScheme},
    widget::{
        Action,
        button::{self, Content},
        button_group::{self, standard::GroupItem},
        connected_button_group,
        hybrid_icon::Icon,
        menu::{self, button::MenuAction},
        slider, standard_button_group,
    },
};
use iced_widget::{column, container, mouse_area, responsive, row, space, text};

use crate::gui::{
    Chilen, ROUNDING_LARGE, SPACING_REGULAR, SPACING_SMALL, SPACING_SMALLER, dialog,
    event_loop::{Message, library},
    font::{self, bold_text},
    formatter::{UNKNOWN_TRACK_DURATION, format_track_duration},
    icons,
    main_view::{
        self,
        top_view::{self, TopView},
    },
    playback_view,
    widget::{cover_image::CoverImage, menu_button::MenuButton, wrappers::marquee},
};

pub fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
    let cover = container(responsive(|size| {
        let cover_size = size.width;
        let image_path = state
            .player_state
            .as_ref()
            .and_then(|s| s.current().and_then(|track| track.cover.cover().ok()));

        CoverImage {
            image_path,
            icon: *icons::MUSIC_NOTE,
            icon_size: cover_size / 4.0,
            icon_color: state.theme.on_surface_variant(),
            container_color: state.theme.surface_container_high(),
            radius: ROUNDING_LARGE.into(),
            opacity: 1.0,
            width: cover_size.into(),
            height: cover_size.into(),
        }
        .into()
    }))
    .height(Length::Shrink);

    let current_track = state.player_state.as_ref().and_then(|p| p.current());
    let title = if let Some(track) = current_track.as_ref() {
        track
            .title
            .as_ref()
            .map(|t| t.clone().into_string())
            .unwrap_or("Untitled".to_string())
    } else {
        "Nothing playing".to_string()
    };

    let title = marquee(
        bold_text(title)
            .size(32.0)
            .wrapping(text::Wrapping::None)
            .color(state.theme.on_surface()),
    )
    .animation(chilen_widget::marquee::Animation::Always);

    let artists: Element<'_, Message> = if let Some(track) = current_track.as_ref()
        && let Some(artists) = &track.artists
        && !artists.is_empty()
        && let Some(lib) = state.library.as_ref()
    {
        let artists: Vec<_> = artists
            .into_iter()
            .filter_map(|a| lib.find_artist(a))
            .map(|a| {
                mouse_area(
                    text(a.name.clone().into_string())
                        .wrapping(text::Wrapping::None)
                        .size(font::SIZE_REGULAR)
                        .color(state.theme.on_surface_variant()),
                )
                .on_press(Message::MainView(main_view::Message::TopView(
                    top_view::Message::Navigate(TopView::Artist(a.clone())),
                )))
                .interaction(iced::mouse::Interaction::Pointer)
                .into()
            })
            .collect();

        let mut artists_w_separators = Vec::with_capacity(artists.len() * 2 - 1);
        let len = artists.len();
        for (i, artist) in artists.into_iter().enumerate() {
            artists_w_separators.push(artist);
            if i < len - 1 {
                artists_w_separators.push(
                    text(&state.settings.value_separator)
                        .wrapping(text::Wrapping::None)
                        .size(font::SIZE_REGULAR)
                        .color(state.theme.on_surface_variant())
                        .into(),
                );
            }
        }

        row(artists_w_separators).into()
    } else {
        text("Unknown artist")
            .wrapping(text::Wrapping::None)
            .size(font::SIZE_REGULAR)
            .color(state.theme.on_surface_variant())
            .into()
    };
    let artists = marquee(artists);

    // TODO: Can't do this fancy thing as I can't set the exact button content
    // let album = if let Some(state) = &state.player_state
    //     && let Some(track) = state.current()
    // {
    //     track.album.clone()
    // } else {
    //     None
    // };
    // let album_text = album
    //     .as_ref()
    //     .map(|a| a.clone().into_string())
    //     .unwrap_or("Unknown album".to_string());
    // let album_button = iced_m3::widget::button(
    //     button::Style::tonal(&state.theme, Accent::Tertiary),
    //     Content::Full {
    //         icon: Icon::Text {
    //             text: icons::ALBUM.into_fragment(),
    //             font: Some(icons::filled()),
    //         },
    //         label: album_text.into(),
    //     },
    // )
    // .on_press_maybe(album.map(|a| Message::OpenAlbum(a.into_string())))
    // .size(button::Size::extra_small());

    let album: Element<'_, Message> = if let Some(track) = current_track.as_ref()
        && let Some(album) = &track.album
        && let Some(lib) = state.library.as_ref()
    {
        let album = lib.find_album(album);

        let area = mouse_area(
            text(
                album
                    .as_ref()
                    .map(|a| a.title.clone().into_string())
                    .unwrap_or("Unknown album".to_string()),
            )
            .wrapping(text::Wrapping::None)
            .size(font::SIZE_REGULAR)
            .color(state.theme.on_surface_variant()),
        )
        .interaction(iced::mouse::Interaction::Pointer);

        match album {
            Some(album) => area.on_press(Message::MainView(main_view::Message::TopView(
                top_view::Message::Navigate(TopView::Album(album.clone())),
            ))),
            None => area,
        }
        .into()
    } else {
        text("Unknown album")
            .wrapping(text::Wrapping::None)
            .size(font::SIZE_REGULAR)
            .color(state.theme.on_surface_variant())
            .into()
    };
    let album = marquee(album);

    // TODO: Maybe could be moved to the connected button row?
    let menu_button = MenuButton::new(
        move || {
            let items = if let Some(track) = current_track {
                vec![
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::PLAYLIST_ADD.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Add to playlist".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: Some(Action::Direct(Message::Dialog(
                                dialog::Message::AddTracksToPlaylist(vec![track.clone()]),
                            ))),
                        },
                    }),
                    menu::Item::Button(menu::button::Button {
                        icon: Some(Icon::Text {
                            text: icons::INFO.into_fragment(),
                            font: Some(icons::filled()),
                        }),
                        label: "Details".into(),
                        supporting_text: None,
                        action: MenuAction::Message {
                            is_selected: false,
                            on_press: None,
                        },
                    }),
                ]
            } else {
                Vec::new()
            };

            iced_m3::widget::menu(
                menu::Style::new(&state.theme, state.settings.vibrant_widgets),
                vec![menu::Group::new(items)],
            )
            .into()
        },
        &state.theme,
        current_track.is_some().then_some(Message::PlaybackView(
            playback_view::Message::ToggleMainMenu,
        )),
    )
    .opened(current_track.is_some() && state.playback_view.main_menu_opened);

    let content = row![
        column![title, artists, album].width(Length::Fill),
        menu_button
    ]
    .spacing(SPACING_SMALL)
    .align_y(Alignment::Center);

    let max_value = 1000;

    let (player_position, track_duration) = state
        .player_state
        .as_ref()
        .map(|p| {
            if let Some(track) = p.current() {
                (p.player_position, Some(track.duration))
            } else {
                (p.player_position, None)
            }
        })
        .unwrap_or((Duration::default(), None));

    let value = if let Some(held) = state.playback_view.seek_slider_value {
        (held * max_value as f32) as u32
    } else {
        if let Some(track_duration) = track_duration {
            if track_duration.as_secs_f32() != 0.0 {
                ((player_position.as_secs_f32() / track_duration.as_secs_f32()) * max_value as f32)
                    as u32
            } else {
                0
            }
        } else {
            0
        }
    };

    let slider = iced_m3::widget::slider(
        0..=max_value,
        value,
        move |position| {
            Message::PlaybackView(playback_view::Message::SeekSliderMoved(if max_value != 0 {
                position as f32 / max_value as f32
            } else {
                0.0
            }))
        },
        slider::Style::new(&state.theme),
    )
    .on_release(Message::PlaybackView(
        playback_view::Message::SeekSliderReleased,
    ));

    let player_position = track_duration
        .map(|d| {
            let duration = state
                .playback_view
                .seek_slider_value
                .map(|v| Duration::from_secs_f32(d.as_secs_f32() * v))
                .unwrap_or(player_position);
            format_track_duration(duration)
        })
        .unwrap_or(UNKNOWN_TRACK_DURATION.to_string());
    let (player_pos_color, player_pos_font) = state
        .playback_view
        .seek_slider_value
        .map(|_| (state.theme.on_surface(), font::bold()))
        .unwrap_or((state.theme.on_surface_variant(), font::regular()));
    let track_duration = track_duration
        .map(format_track_duration)
        .unwrap_or(UNKNOWN_TRACK_DURATION.to_string());

    let timestamps = row![
        text(player_position)
            .size(font::SIZE_SMALL)
            .color(player_pos_color)
            .font(player_pos_font),
        space().width(Length::Fill),
        text(track_duration)
            .size(font::SIZE_SMALL)
            .color(state.theme.on_surface_variant()),
    ];

    let slider_stuff = column![slider, timestamps].spacing(SPACING_SMALLER);

    let play_button_icon = state
        .player_state
        .as_ref()
        .map(|p| match p.playback_state {
            chilen_backend::playback::PlaybackState::Playing => *icons::PAUSE,
            chilen_backend::playback::PlaybackState::Paused => *icons::PLAY_ARROW,
            chilen_backend::playback::PlaybackState::Stopped => *icons::STOP,
        })
        .unwrap_or(*icons::STOP);

    let skip_button_size = {
        let size = iced_m3::widget::button::Size::medium();
        size.padding((size.padding.left * 1.3).into())
    };
    let top_button_row = vec![
        {
            let message_maybe = state.player_state.as_ref().and_then(|p| {
                p.can_go_previous()
                    .then_some(Message::PlaybackView(playback_view::Message::Previous))
            });
            let is_some = message_maybe.is_some();

            GroupItem::new(
                iced_m3::widget::button(
                    button::Style::tonal(&state.theme, Accent::Tertiary),
                    Content::Icon(Icon::Text {
                        text: icons::SKIP_PREVIOUS.into_fragment(),
                        font: Some(icons::filled()),
                    }),
                )
                .size(skip_button_size)
                .on_press_maybe(message_maybe),
                is_some,
            )
        },
        {
            let message_maybe = state.player_state.as_ref().and_then(|p| {
                p.can_toggle_playing()
                    .then_some(Message::PlaybackView(playback_view::Message::TogglePlaying))
            });
            let is_some = message_maybe.is_some();

            GroupItem::new(
                iced_m3::widget::button(
                    button::Style::tonal(&state.theme, Accent::Primary),
                    Content::Icon(Icon::Text {
                        text: play_button_icon.into_fragment(),
                        font: Some(icons::filled()),
                    }),
                )
                .size(button::Size::medium().width(Length::Fill))
                .selected(
                    state
                        .player_state
                        .as_ref()
                        .map(|p| p.is_playing())
                        .unwrap_or_default(),
                )
                .on_press_maybe(message_maybe),
                is_some,
            )
        },
        {
            let message_maybe = state.player_state.as_ref().and_then(|p| {
                p.can_go_next()
                    .then_some(Message::PlaybackView(playback_view::Message::Next))
            });
            let is_some = message_maybe.is_some();

            GroupItem::new(
                iced_m3::widget::button(
                    button::Style::tonal(&state.theme, Accent::Tertiary),
                    Content::Icon(Icon::Text {
                        text: icons::SKIP_NEXT.into_fragment(),
                        font: Some(icons::filled()),
                    }),
                )
                .size(skip_button_size)
                .on_press_maybe(message_maybe),
                is_some,
            )
        },
    ];

    let connected_buttons = vec![
        {
            let (message_maybe, selected) = match state.player_state.as_ref() {
                Some(state) => (
                    Some(Message::PlaybackView(playback_view::Message::ToggleShuffle)),
                    state.shuffle_enabled(),
                ),
                None => (None, false),
            };

            button_group::connected::Button {
                content: Content::Icon(Icon::Text {
                    text: icons::SHUFFLE.into_fragment(),
                    font: Some(icons::filled()),
                }),
                on_press: message_maybe,
                selected,
            }
        },
        {
            let message_maybe = state
                .player_state
                .as_ref()
                .map(|_| Message::PlaybackView(playback_view::Message::ToggleLooping));
            let (loop_button_icon, selected) = state
                .player_state
                .as_ref()
                .map(|p| match p.loop_state {
                    LoopState::Off => (*icons::REPEAT, false),
                    LoopState::Playlist => (*icons::REPEAT, true),
                    LoopState::Track => (*icons::REPEAT_ONE, true),
                })
                .unwrap_or((*icons::REPEAT, false));

            button_group::connected::Button {
                content: button::Content::Icon(Icon::Text {
                    text: loop_button_icon.into_fragment(),
                    font: Some(icons::filled()),
                }),
                on_press: message_maybe,
                selected,
            }
        },
        {
            let favorite_item: Option<LibraryItem> = current_track.map(|t| t.clone().into());
            let is_favorite = favorite_item
                .as_ref()
                .map(|f| is_favorite(f).unwrap_or_default())
                .unwrap_or_default();
            let icon_font = match is_favorite {
                true => icons::filled(),
                false => icons::outlined(),
            };

            button_group::connected::Button {
                content: button::Content::Icon(Icon::Text {
                    text: icons::STAR.into_fragment(),
                    font: Some(icon_font),
                }),
                on_press: favorite_item
                    .map(|f| Message::Library(library::Message::ToggleFavorite(f))),
                selected: is_favorite,
            }
        },
    ];

    let connected_buttons = connected_button_group(
        button::Style::tonal(&state.theme, Accent::Secondary),
        connected_buttons,
    );

    let buttons = column![
        standard_button_group(top_button_row).spacing(button_group::standard::SPACING_MEDIUM),
        connected_buttons
    ]
    .spacing(SPACING_SMALL)
    .align_x(Alignment::Center);

    column![
        cover,
        content,
        column![slider_stuff, buttons].spacing(SPACING_REGULAR)
    ]
    .spacing(SPACING_REGULAR)
    .into()
}
