mod about;
mod library;
mod look_and_feel;
mod playback;

use iced::{Alignment, Border, Element, Length, Pixels, Task, color};
use iced_core::text::{IntoFragment, LineHeight::Absolute};
use iced_m3::{
    style::{Elevation, shadow},
    theme::{Accent, ColorScheme},
    widget::{
        OnPress, button,
        fab::{self},
        navrail::{self, CONTAINER_EXPANDED_MIN_WIDTH, Item},
    },
};
use iced_widget::{center, column, container, opaque, row, stack, text};

use crate::gui::{
    Chilen, ROUNDING_REGULAR, SPACING_REGULAR,
    dialog::Dialog,
    font::bold_text,
    icons::{self, INFO, LIBRARY_MUSIC, PALETTE, PLAY_ARROW},
};

const MAX_WIDTH: f32 = 1000.0;

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
    Reset,
}

#[derive(Default)]
pub struct State {
    screen: Screen,
}

pub(super) fn update(state: &mut Chilen, message: Message) -> Task<Message> {
    match message {
        Message::Close => state.settings_opened = false,
        Message::SwitchScreen(screen) => state.settings_state.screen = screen,
        Message::LookAndFeel(message) => {
            return look_and_feel::update(state, message).map(Message::LookAndFeel);
        }
        Message::Reset => state.dialog = Dialog::ResetSettings,
    }
    Task::none()
}

pub(super) fn view<'a>(state: &'a Chilen) -> Element<'a, Message> {
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
    .size(button::Size::Medium)
    .on_press(Message::Close);

    let items = [
        Screen::LookAndFeel,
        Screen::Library,
        Screen::Playback,
        Screen::About,
    ]
    .into_iter()
    .map(|screen| Item {
        icon: iced_m3::widget::BadgeIcon {
            icon: screen.icon().into_fragment(),
            badge: None,
        },
        label: screen.label(),
        on_press: OnPress::Direct(Message::SwitchScreen(screen)),
    })
    .collect();

    let active_index = state.settings_state.screen.index();
    let navrail = iced_m3::widget::navrail(&state.theme, items)
        .status(navrail::Status::Expanded {
            width: Pixels(CONTAINER_EXPANDED_MIN_WIDTH),
        })
        .fab(navrail::Fab {
            icon: icons::RESET_SETTINGS.into_fragment(),
            label: "Reset settings".into(),
            style: fab::Style::fab_tonal(&state.theme, Accent::Tertiary),
            on_press: OnPress::Direct(Message::Reset),
        })
        .icon_font(icons::filled())
        .container_vertical_padding(iced_m3::widget::navrail::ITEM_OFFSET)
        .icon_font_active(icons::filled())
        .icon_font_inactive(icons::outlined())
        .active(active_index);

    let title = bold_text(state.settings_state.screen.label())
        .size(32.0)
        .line_height(Absolute(Pixels(32.0)))
        .color(state.theme.on_surface());
    let content_padding = SPACING_REGULAR;
    let settings_page =
        column![title, state.settings_state.screen.view(state)].spacing(SPACING_REGULAR);
    let content = row![
        container(navrail).style(move |_| container::Style::default()
            .background(state.theme.surface_container())
            .border(Border::default().rounded(rounding))),
        container(
            stack![
                settings_page,
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
                .border(Border::default().rounded(rounding))
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
}
