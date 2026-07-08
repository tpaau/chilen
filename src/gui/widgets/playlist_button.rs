use std::{str::FromStr, sync::Arc};

use iced::{
    Background, Border, Color, Font, Length, Padding, Shadow,
    font::Weight,
    widget::{Button, Space, button, column, container, row, space, text},
};

use crate::{
    gui::{Chilen, playlist_view},
    music_lib::state::Playlist,
};

const THUMBNAIL_SIZE: f32 = 64.0;

pub fn playlist_button<'a>(
    state: &'a Chilen,
    playlist: &'a Arc<Playlist>,
) -> Button<'a, playlist_view::Message> {
    button(
        row(vec![
            container(Space::new())
                .style(|_| {
                    container::Style::default()
                        .background(Color::from_str("#FF0000").unwrap())
                        .border(
                            Border::default()
                                .rounded(state.rounding.regular - state.spacing.smaller),
                        )
                })
                .width(Length::Fixed(THUMBNAIL_SIZE))
                .height(Length::Fixed(THUMBNAIL_SIZE))
                .into(),
            container(column(vec![
                text(playlist.name.clone())
                    .size(state.font_size.regular)
                    .color(state.theme.current().on_surface)
                    .font(Font {
                        weight: Weight::Semibold,
                        ..Default::default()
                    })
                    .into(),
                text({
                    if playlist.tracks.is_empty() {
                        "No tracks".to_string()
                    } else {
                        format!("{} tracks", playlist.tracks.len())
                    }
                })
                .size(state.font_size.small)
                .color(state.theme.current().on_surface)
                .into(),
            ]))
            .center_y(Length::Fill)
            .into(),
            space().width(Length::Fill).into(),
            container(text("aaa")).center_y(Length::Fill).into(),
        ])
        .spacing(state.spacing.small),
    )
    .padding(Padding::new(state.spacing.smaller as f32))
    .style(|_, status| {
        let style = button::Style {
            background: Some(Background::Color(
                state.theme.current().surface_container_low,
            )),
            text_color: state.theme.current().on_surface,
            border: Border::default().rounded(state.rounding.regular),
            shadow: Shadow::default(),
            snap: true,
        };

        match status {
            button::Status::Hovered => style.with_background(Background::Color(
                state.theme.current().surface_container_high,
            )),
            _ => style,
        }
    })
    .on_press(playlist_view::Message::Open(playlist.clone()))
}
