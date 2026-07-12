use std::{str::FromStr, sync::Arc};

use iced::{
    Background, Border, Color, Length, Padding, Shadow, Vector,
    widget::{Button, Space, button, column, container, row, space, text},
};

use crate::{
    gui::{
        Chilen, DIM_TEXT_ALPHA, ROUNDING_REGULAR, SPACING_SMALL, SPACING_SMALLER,
        font::{self, SIZE_REGULAR, SIZE_SMALL},
        icons, playlist_view,
        theme::ColorScheme,
        widgets::common::drop_down_menu::DropDownMenu,
    },
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
                        .border(Border::default().rounded(ROUNDING_REGULAR - SPACING_SMALLER))
                })
                .width(Length::Fixed(THUMBNAIL_SIZE))
                .height(Length::Fixed(THUMBNAIL_SIZE))
                .into(),
            container(column(vec![
                text(playlist.name.clone())
                    .size(SIZE_REGULAR)
                    .color(state.theme.on_surface())
                    .into(),
                text({
                    if playlist.tracks.is_empty() {
                        "No tracks".to_string()
                    } else {
                        format!("{} tracks", playlist.tracks.len())
                    }
                })
                .size(SIZE_SMALL)
                .color(state.theme.on_surface().scale_alpha(DIM_TEXT_ALPHA))
                .into(),
            ]))
            .center_y(Length::Fill)
            .into(),
            space().width(Length::Fill).into(),
            container(
                // TODO: Should be more like a button
                DropDownMenu::new(
                    text(*icons::MORE_HORIZ)
                        .font(icons::font())
                        .size(icons::SIZE_REGULAR),
                    container(column![
                        text("Option 1").size(font::SIZE_REGULAR),
                        text("Option 2").size(font::SIZE_REGULAR),
                        text("Option 3").size(font::SIZE_REGULAR),
                        text("Option 4").size(font::SIZE_REGULAR),
                    ])
                    .style(|_| {
                        container::Style::default()
                            .background(state.theme.surface_container_low())
                            .border(Border::default().rounded(16))
                            .shadow(Shadow {
                                color: state.theme.shadow().scale_alpha(0.6),
                                offset: Vector::new(0.0, 2.0),
                                blur_radius: 6.0,
                            })
                    })
                    .padding(Padding::from(SPACING_SMALL as f32)),
                ),
            )
            .center_y(Length::Fill)
            .into(),
        ])
        .spacing(SPACING_SMALL),
    )
    .padding(Padding::new(SPACING_SMALLER as f32))
    .style(|_, status| {
        let style = button::Style {
            background: Some(Background::Color(state.theme.surface_container())),
            text_color: state.theme.on_surface(),
            border: Border::default().rounded(ROUNDING_REGULAR),
            shadow: Shadow::default(),
            snap: true,
        };

        match status {
            button::Status::Hovered => {
                style.with_background(Background::Color(state.theme.surface_container_high()))
            }
            _ => style,
        }
    })
    .on_press(playlist_view::Message::Open(playlist.clone()))
}
