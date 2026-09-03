use std::borrow::Cow;

use iced::{Alignment, Border, Element, Font, Length, color};
use iced_widget::{column, container, opaque, row, space, text::IntoFragment};

use crate::theme::ColorScheme;

pub const MIN_WIDTH: f32 = 280.0;
pub const MAX_WIDTH: f32 = 560.0;
const DIALOG_RADIUS: f32 = 24.0;
const ICON_SIZE: f32 = 24.0;
const TITLE_SIZE: f32 = 24.0;
const PADDING: f32 = 24.0;
const ICON_TITLE_SPACING: f32 = 16.0;
const TITLE_BODY_SPACING: f32 = 16.0;
const BODY_BUTTONS_SPACING: f32 = 24.0;
const BUTTON_SPACING: f32 = 8.0;

pub struct Button<Message> {
    pub on_press: Option<Message>,
    pub label: String,
    pub style: crate::widget::button::Style,
}

pub struct Dialog<'a, Message> {
    icon: Option<char>,
    icon_font: Option<Font>,
    title: Cow<'a, str>,
    title_font: Option<Font>,
    body: Element<'a, Message>,
    theme: &'a dyn ColorScheme,
    buttons: Vec<Button<Message>>,
    button_label_font: Option<Font>,
    width: Option<f32>,
    height: Option<f32>,
}

impl<'a, Message> Dialog<'a, Message> {
    #[must_use]
    pub fn new(
        theme: &'a impl ColorScheme,
        body: impl Into<Element<'a, Message>>,
        buttons: Vec<Button<Message>>,
    ) -> Self {
        Self {
            icon: None,
            icon_font: None,
            title: Cow::Borrowed("Title"),
            title_font: None,
            body: body.into(),
            theme,
            buttons,
            button_label_font: None,
            width: None,
            height: None,
        }
    }

    #[must_use]
    pub fn icon(mut self, icon: char) -> Self {
        self.icon = Some(icon);
        self
    }

    #[must_use]
    pub fn icon_maybe(mut self, maybe_icon: Option<char>) -> Self {
        self.icon = maybe_icon;
        self
    }

    #[must_use]
    pub fn icon_font(mut self, font: Font) -> Self {
        self.icon_font = Some(font);
        self
    }

    #[must_use]
    pub fn icon_font_maybe(mut self, font: Option<Font>) -> Self {
        self.icon_font = font;
        self
    }

    #[must_use]
    pub fn title(mut self, title: impl IntoFragment<'a>) -> Self {
        self.title = title.into_fragment();
        self
    }

    #[must_use]
    pub fn title_font(mut self, font: Font) -> Self {
        self.title_font = Some(font);
        self
    }

    #[must_use]
    pub fn title_font_maybe(mut self, font: Option<Font>) -> Self {
        self.title_font = font;
        self
    }

    #[must_use]
    pub fn button_label_font(mut self, font: Font) -> Self {
        self.button_label_font = Some(font);
        self
    }

    #[must_use]
    pub fn button_label_font_maybe(mut self, font: Option<Font>) -> Self {
        self.button_label_font = font;
        self
    }

    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    #[must_use]
    pub fn width_maybe(mut self, maybe_width: Option<f32>) -> Self {
        self.width = maybe_width;
        self
    }

    #[must_use]
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    #[must_use]
    pub fn height_maybe(mut self, maybe_height: Option<f32>) -> Self {
        self.height = maybe_height;
        self
    }
}

impl<'a, Message> From<Dialog<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: Dialog<'a, Message>) -> Self {
        let icon = value.icon.map(|i| {
            column![
                iced_widget::text(i)
                    .size(ICON_SIZE)
                    .color(value.theme.secondary())
                    .font_maybe(value.icon_font),
                space().height(ICON_TITLE_SPACING)
            ]
        });

        let title = column![
            iced_widget::text(value.title)
                .size(TITLE_SIZE)
                .color(value.theme.on_surface())
                .font_maybe(value.title_font),
            space().height(TITLE_BODY_SPACING)
        ];

        let buttons = row({
            let mut content = Vec::with_capacity(value.buttons.len() + 1);
            content.push(space().width(Length::Fill).into());
            content.extend(value.buttons.into_iter().map(|b| {
                crate::widget::button(value.theme)
                    .label(b.label)
                    .label_font_maybe(value.button_label_font)
                    .style(b.style)
                    .on_press_maybe(b.on_press)
                    .into()
            }));
            content
        })
        .spacing(BUTTON_SPACING);

        let alignment = if value.icon.is_some() {
            Alignment::Center
        } else {
            Alignment::Start
        };

        let column = column![
            // A little hack to enforce the min width
            space().width(MIN_WIDTH),
            icon,
            title,
            value.body,
            space().height(BODY_BUTTONS_SPACING),
            buttons
        ]
        .align_x(alignment);

        let dialog = container(column)
            .padding(PADDING)
            .style(|_| {
                iced_widget::container::Style::default()
                    .background(value.theme.surface_container_high())
                    .border(Border::default().rounded(DIALOG_RADIUS))
                    .shadow(crate::style::shadow(value.theme.shadow(), 0.6))
            })
            .max_width(MAX_WIDTH);

        let dialog = match value.width {
            Some(width) => dialog.width(width.clamp(MIN_WIDTH, MAX_WIDTH)),
            None => dialog,
        };

        let dialog = match value.height {
            Some(height) => dialog.height(height),
            None => dialog,
        };

        opaque(
            container(dialog)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .padding(PADDING)
                .style(|_| {
                    iced_widget::container::Style::default()
                        .background(color!(0x000000).scale_alpha(0.3))
                }),
        )
    }
}
