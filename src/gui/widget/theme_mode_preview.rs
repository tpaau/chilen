use iced::{Alignment, Border, Element, Length, Padding, Pixels, border::Radius};
use iced_core::text::LineHeight;
use iced_m3::{
    style::DISABLED_STATE_LAYER_OPACITY,
    theme::ColorScheme,
    widget::{OnPress, card::Interaction, icon, progress_bar},
};
use iced_widget::{column, container, row, space};

use crate::gui::{
    self, ROUNDING_LARGE, ROUNDING_SMALL, SPACING_REGULAR, SPACING_SMALL,
    font::{self, bold_text},
    icons,
};

pub struct ThemeModePreview<'a, Message>
where
    Message: Clone,
{
    pub theme: &'a dyn ColorScheme,
    pub label: &'a str,
    pub selected: bool,
    pub on_press: Option<OnPress<'a, Message>>,
}

impl<'a, Message> From<ThemeModePreview<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: ThemeModePreview<'a, Message>) -> Self {
        let radius = ROUNDING_LARGE;
        let padding = 8.0;
        let opacity = match value.on_press.is_some() {
            true => 1.0,
            false => DISABLED_STATE_LAYER_OPACITY,
        };

        let check_icon = value.selected.then_some(
            icon(*icons::CHECK, icons::SIZE_LARGE)
                .font(icons::filled())
                .color(value.theme.on_primary_container().scale_alpha(opacity)),
        );

        let label = bold_text(value.label)
            .color(value.theme.on_surface().scale_alpha(opacity))
            .size(font::SIZE_LARGE)
            .line_height(LineHeight::Absolute(Pixels(icons::SIZE_LARGE)));

        let icon_visible = check_icon.is_some();
        let container_rounding_small = 4.0;
        let preview = column![
            row![
                container(
                    icon(*icons::PERSON, 50.0 - 2.0 * padding)
                        .font(icons::filled())
                        .color(value.theme.on_surface().scale_alpha(opacity))
                )
                .padding(padding)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .style(move |_| {
                    container::Style::default()
                        .background(value.theme.surface_container_high().scale_alpha(opacity))
                        .border(Border::default().rounded(f32::MAX))
                }),
                column![
                    container(space().height(25).width(Length::Fill)).style(move |_| {
                        container::Style::default()
                            .background(value.theme.surface_container_high().scale_alpha(opacity))
                            .border(Border::default().rounded(ROUNDING_SMALL))
                    }),
                    container(
                        container(space().height(25.0 - padding).width(Length::Fill)).style(
                            move |_| {
                                container::Style::default()
                                    .background(
                                        value.theme.surface_container_high().scale_alpha(opacity),
                                    )
                                    .border(Border::default().rounded(ROUNDING_SMALL))
                            }
                        )
                    )
                    .padding(Padding::default().right(60))
                ]
                .spacing(padding),
            ]
            .spacing(padding),
            progress_bar(progress_bar::Style {
                bar_color: value.theme.secondary_container().scale_alpha(opacity),
                track_color: value.theme.primary().scale_alpha(opacity),
                stop_indicator_color: value.theme.primary().scale_alpha(opacity)
            }),
            row![
                container(check_icon)
                    .width(Length::Fill)
                    .height(40)
                    .style(move |_| {
                        container::Style::default()
                            .background(
                                if icon_visible {
                                    value.theme.primary_container()
                                } else {
                                    value.theme.surface_container_high()
                                }
                                .scale_alpha(opacity),
                            )
                            .border(Border::default().rounded(f32::MAX))
                    })
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center),
                container(space().width(Length::Fill).height(40)).style(move |_| {
                    container::Style::default()
                        .background(
                            if icon_visible {
                                value.theme.tertiary_container()
                            } else {
                                value.theme.surface_container_high()
                            }
                            .scale_alpha(opacity),
                        )
                        .border(Border::default().rounded(container_rounding_small))
                }),
                container(space().width(Length::Fill).height(40)).style(move |_| {
                    container::Style::default()
                        .background(
                            if icon_visible {
                                value.theme.tertiary_container()
                            } else {
                                value.theme.surface_container_high()
                            }
                            .scale_alpha(opacity),
                        )
                        .border(
                            Border::default().rounded(
                                Radius::default()
                                    .left(container_rounding_small)
                                    .right(f32::MAX),
                            ),
                        )
                })
            ]
            .spacing(container_rounding_small)
        ]
        .spacing(SPACING_REGULAR);

        let container = container(preview)
            .style(move |_| {
                container::Style::default()
                    .border(Border::default().rounded(radius - padding))
                    .background(value.theme.surface().scale_alpha(opacity))
            })
            .padding(padding);

        let container_label_spacing = SPACING_SMALL;
        let content = column![container, label]
            .spacing(container_label_spacing)
            .align_x(Alignment::Center);

        iced_m3::widget::card(gui::settings::card_style(value.theme), content)
            .width(Length::Fill)
            .interaction(
                value
                    .on_press
                    .map(|on_press| Interaction::Press(on_press))
                    .unwrap_or(Interaction::Disabled),
            )
            .padding(Padding::from(padding))
            .into()
    }
}
