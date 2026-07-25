use iced::{
    Alignment, Border, Element, Length, Padding,
    border::Radius,
    widget::{column, container, text},
};
use iced_m3::{
    style::shadow,
    theme::ColorScheme,
    widget::drop_down_menu::{DropDownMenu, Placement},
};
use iced_widget::{bottom_right, center, row, space, stack};

use crate::gui::{
    self, Chilen, LoadingState, Message, ROUNDING_LARGE, ROUNDING_REGULAR, SPACING_SMALL,
    SPACING_SMALLER, font, icons, widgets::playlist_button::playlist_button,
};

pub fn view(state: &Chilen) -> Element<'_, Message> {
    match &state.loading_state {
        LoadingState::Loading => text!("Loading...").color(state.theme.on_surface()).into(),
        LoadingState::Failed(e) => {
            container(text!("Load failed: {e}").color(state.theme.on_error()))
                .style(|_| {
                    container::Style::default()
                        .background(state.theme.error_container())
                        .border(Border::default().rounded(ROUNDING_REGULAR))
                })
                .width(Length::Fill)
                .padding(Padding::new(SPACING_SMALLER as f32))
                .into()
        }
        LoadingState::Loaded => {
            stack!(
                column![
                    text!("Playlists")
                        .color(state.theme.on_surface())
                        .size(font::SIZE_LARGE)
                        .font(gui::font::font_bold()),
                    iced::widget::scrollable(
                        column({
                            // TODO: Proper sorting with support for numbers and non-ASCII characters
                            let mut playlists: Vec<_> =
                                state.library.as_ref().unwrap().playlists.iter().collect();
                            playlists.sort_by_key(|pl| pl.name.clone());
                            playlists
                                .into_iter()
                                .map(|p| playlist_button(state, p).width(Length::Fill).into())
                        })
                        .spacing(SPACING_SMALLER)
                    )
                    .style(|_, status| iced_m3::style::scrollable(status, &state.theme))
                    .height(Length::Fill)
                    .width(Length::Fill),
                ]
                .spacing(SPACING_SMALL)
                .height(Length::Fill)
                .width(Length::Fill),
                bottom_right(
                    // TODO: The overlay should pass down mouse clicks in transparent mode too
                    DropDownMenu::new(
                        move |opened| container(center(
                            text(if opened { *icons::CLOSE } else { *icons::ADD })
                                .font(icons::font())
                                .size(icons::SIZE_REGULAR)
                        ))
                        .style(move |_| container::Style {
                            text_color: Some(if opened {
                                state.theme.primary()
                            } else {
                                state.theme.on_primary()
                            }),
                            background: Some(iced::Background::Color(if opened {
                                state.theme.on_primary()
                            } else {
                                state.theme.primary()
                            })),
                            border: Border::default().rounded(if opened {
                                Radius::from(u32::MAX)
                            } else {
                                Radius::from(ROUNDING_LARGE)
                            }),
                            shadow: shadow(state.theme.shadow(), 0.6),
                            snap: true
                        })
                        .width(Length::Fixed(56.0))
                        .height(Length::Fixed(56.0))
                        .into(),
                        Some(
                            column![
                                iced::widget::button(center(
                                    row(vec![
                                        text(*icons::UPLOAD_FILE)
                                            .font(icons::font())
                                            .size(icons::SIZE_SMALLER)
                                            .into(),
                                        text("Import playlist").size(font::SIZE_REGULAR).into()
                                    ])
                                    .align_y(Alignment::Center)
                                    .spacing(8)
                                ))
                                .style(|_, status| {
                                    let mut style = iced_m3::style::button(
                                        status,
                                        &state.theme,
                                        iced_m3::style::Button::Primary,
                                    );
                                    style.border.radius = Radius::from(f32::MAX);
                                    style
                                })
                                .padding(Padding::from(16.0))
                                .height(Length::Fixed(56.0))
                                .width(Length::Shrink)
                                .on_press(Message::OpenPlaylistImportFilePicker),
                                iced::widget::button(center(
                                    row(vec![
                                        text(*icons::PLAYLIST_ADD)
                                            .font(icons::font())
                                            .size(icons::SIZE_SMALLER)
                                            .into(),
                                        text("New playlist").size(font::SIZE_REGULAR).into()
                                    ])
                                    .align_y(Alignment::Center)
                                    .spacing(8)
                                ))
                                .style(|_, status| {
                                    let mut style = iced_m3::style::button(
                                        status,
                                        &state.theme,
                                        iced_m3::style::Button::Primary,
                                    );
                                    style.border.radius = Radius::from(f32::MAX);
                                    style
                                })
                                .padding(Padding::from(16.0))
                                .height(Length::Fixed(56.0))
                                .width(Length::Shrink)
                                .on_press(Message::OpenPlaylistCreationDialog),
                                space().height(4.0)
                            ]
                            .align_x(Alignment::End)
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .spacing(4)
                        ),
                        Placement::TopLeft,
                    )
                    .transparent(true)
                )
            )
            .into()
        }
    }
}
