mod about;
mod library;
mod look_and_feel;
mod playback;

use std::{sync::LazyLock, thread};

use iced::{Alignment, Border, Element, Length, Pixels, Task, color};
use iced_core::{
    image::Handle,
    text::{IntoFragment, LineHeight::Absolute},
};
use iced_m3::{
    style::{Elevation, shadow},
    theme::{Accent, ColorScheme},
    widget::{
        OnPress, button, card,
        fab::{self},
        hybrid_icon::Icon,
        navrail::{self, CONTAINER_EXPANDED_MIN_WIDTH, Item},
        shapes::polymorpher::{
            RoundedPolygon,
            shapes::{
                clover4, clover8, cookie4, cookie6, cookie7, cookie9, cookie12, sunny, very_sunny,
            },
        },
        switch,
    },
};
use iced_widget::{center, column, container, opaque, row, stack, text};
use log::error;

use crate::gui::{
    Chilen, GOTH_CORNER_ROUNDING, ROUNDING_LARGE, ROUNDING_REGULAR, SPACING_REGULAR,
    dialog::Dialog,
    font::{self, bold_text},
    icons::{self, INFO, LIBRARY_MUSIC, PALETTE, PLAY_ARROW},
};

const MAX_WIDTH: f32 = 1000.0;
const PFP_IMAGE: &[u8] = include_bytes!("../../../resources/images/pfp.png");
pub(super) const TITLE_CONTENT_SPACE: f32 = SPACING_REGULAR;
pub(super) const CARD_SPACING: f32 = iced_m3::widget::card::MAX_CARD_BETWEEN_PADDING;
pub(super) const ITEM_SPACING: f32 = 2.0 * CARD_SPACING;
pub(super) static CLOSE_BUTTON_SIZE: LazyLock<button::Size> = LazyLock::new(button::Size::medium);

#[repr(usize)]
#[derive(Debug, Default, Clone, Copy)]
pub enum Screen {
    #[default]
    LookAndFeel,
    Library,
    Playback,
    About,
}

impl Screen {
    fn label<'a>(&self) -> text::Fragment<'a> {
        match self {
            Screen::LookAndFeel => "Look and feel",
            Screen::Library => "Library",
            Screen::Playback => "Playback",
            Screen::About => "About",
        }
        .into_fragment()
    }

    fn icon(&self) -> char {
        match self {
            Screen::LookAndFeel => *PALETTE,
            Screen::Library => *LIBRARY_MUSIC,
            Screen::Playback => *PLAY_ARROW,
            Screen::About => *INFO,
        }
    }

    fn index(self) -> usize {
        self as usize
    }

    fn view<'a>(&self, state: &'a Chilen) -> Element<'a, Message> {
        match self {
            Screen::LookAndFeel => look_and_feel::view(state).map(Message::LookAndFeel),
            Screen::Library => library::view(state),
            Screen::Playback => playback::view(state),
            Screen::About => about::view(state),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Close,
    SwitchScreen(Screen),
    LookAndFeel(look_and_feel::Message),
    OpenURL(String),
    Reset,
}

pub struct State {
    pub screen: Option<Screen>,
    random_shape: RoundedPolygon,
    random_accent: Accent,
    pfp_image_handle: Handle,
}

pub static AVAILABLE_SHAPES: LazyLock<[RoundedPolygon; 9]> = LazyLock::new(|| {
    [
        cookie12(),
        cookie4(),
        cookie6(),
        cookie7(),
        cookie9(),
        sunny(),
        very_sunny(),
        clover4(),
        clover8(),
    ]
});
impl Default for State {
    fn default() -> Self {
        let accents = [Accent::Primary, Accent::Secondary, Accent::Tertiary];
        let shapes = &AVAILABLE_SHAPES;
        let random_shape = shapes[rand::random_range(0..=(shapes.len() - 1))].clone();
        let random_accent = accents[rand::random_range(0..=(accents.len() - 1))];

        Self {
            screen: None,
            random_shape,
            random_accent,
            pfp_image_handle: Handle::from_bytes(PFP_IMAGE),
        }
    }
}

pub(super) fn title<'a, Message>(
    title: impl IntoFragment<'a>,
    theme: &(impl ColorScheme + ?Sized),
) -> Element<'a, Message> {
    bold_text(title)
        .size(32.0)
        .line_height(Absolute(Pixels(32.0)))
        .color(theme.on_surface())
        .into()
}

pub(super) fn labeled_switch<'a, Message>(
    label: text::Fragment<'a>,
    description: text::Fragment<'a>,
    theme: &'a (impl ColorScheme + ?Sized),
    toggled: bool,
    message: Message,
) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    row![
        column![
            text(label).size(font::SIZE_LARGE).color(theme.on_surface()),
            text(description)
                .size(font::SIZE_REGULAR)
                .color(theme.on_surface_variant())
        ]
        .width(Length::Fill),
        switch(theme, toggled).on_toggle(message)
    ]
    .align_y(Alignment::Center)
    .into()
}

pub(super) fn card_style(theme: &(impl ColorScheme + ?Sized)) -> card::Style {
    card::Style {
        border: Border::default().rounded(ROUNDING_LARGE),
        ..card::Style::elevated(theme)
    }
}

pub(super) fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Close => state.settings_state.screen = None,
        Message::SwitchScreen(screen) => state.settings_state.screen = Some(screen),
        Message::LookAndFeel(message) => {
            return look_and_feel::update(state, message).map(Message::LookAndFeel);
        }
        Message::Reset => state.dialog = Dialog::ResetSettings,
        Message::OpenURL(url) => {
            thread::spawn(move || {
                if let Err(e) = open::that(&url) {
                    error!("Failed to open URL \"{url}\": {e}");
                }
            });
        }
    }
    Task::none()
}

pub(super) fn view<'a>(state: &'a Chilen) -> Option<Element<'a, Message>> {
    state.settings_state.screen.map(|screen| {
        let rounding = ROUNDING_REGULAR;

        let close_button = button(
            button::Style {
                elevation: button::ElevationStates {
                    shadow_color: state.theme.shadow(),
                    idle: Elevation::Level3,
                    disabled: Elevation::Level0,
                    hover: Elevation::Level3,
                    press: Elevation::Level3,
                },
                ..button::Style::filled(&state.theme, Accent::Primary)
            },
            button::Content::Label("Close".into()),
        )
        .size(*CLOSE_BUTTON_SIZE)
        .on_press(Message::Close);

        let items = [
            Screen::LookAndFeel,
            Screen::Library,
            Screen::Playback,
            Screen::About,
        ]
        .into_iter()
        .map(|screen| Item {
            icon_active: Icon::Text {
                text: screen.icon().into_fragment(),
                font: Some(icons::filled()),
            },
            icon_inactive: Icon::Text {
                text: screen.icon().into_fragment(),
                font: Some(icons::outlined()),
            },
            badge: None,
            label: screen.label(),
            on_press: Message::SwitchScreen(screen),
        })
        .collect();

        let active_index = screen.index();
        let navrail = iced_m3::widget::navrail(navrail::Style::new(&state.theme), items)
            .mode(navrail::Mode::Expanded {
                width: Pixels(CONTAINER_EXPANDED_MIN_WIDTH),
            })
            .fab(navrail::Fab {
                icon: Icon::Text {
                    text: icons::RESET_SETTINGS.into_fragment(),
                    font: Some(icons::filled()),
                },
                label: "Reset settings".into(),
                style: fab::Style::fab_tonal(&state.theme, Accent::Tertiary),
                on_press: OnPress::Direct(Message::Reset),
            })
            .container_vertical_padding(iced_m3::widget::navrail::ITEM_OFFSET)
            .active(active_index);

        let content_padding = SPACING_REGULAR;
        let content = row![
            container(navrail).style(move |_| container::Style::default()
                .background(state.theme.surface_container())
                .border(Border::default().rounded(rounding))),
            container(
                stack![
                    screen.view(state),
                    container(close_button)
                        .padding(iced_m3::widget::fab::EDGE_SPACING - content_padding)
                        .align_right(Length::Fill)
                        .align_bottom(Length::Fill)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
            )
            .style(move |_| {
                container::Style::default()
                    .background(state.theme.surface())
                    .border(Border::default().rounded(GOTH_CORNER_ROUNDING))
            })
            .padding(content_padding)
        ];

        opaque(
            container(
                center(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .max_width(MAX_WIDTH)
                    .style(move |_| {
                        container::Style::default()
                            .background(state.theme.surface_container())
                            .border(Border::default().rounded(rounding))
                            .shadow(shadow(state.theme.shadow(), Elevation::Level3))
                    }),
            )
            .align_x(Alignment::Center)
            .style(|_| container::Style::default().background(color!(0x000000).scale_alpha(0.3)))
            .padding(64),
        )
    })
}
