pub mod dialog;
pub mod event_loop;
mod font;
mod formatter;
mod icons;
mod main_view;
pub(super) mod persistent_state;
mod playback_view;
mod settings;
#[cfg(test)]
mod tests;
pub mod themes;
mod widget;

use std::sync::{Arc, LazyLock, RwLock};

use chilen_backend::{music_lib::MusicLibrary, playback::PlayerState};
use iced::{
    self, Element, Font,
    futures::channel::mpsc,
    widget::{container, row},
    window::{self},
};
use iced_m3::{
    theme::{ColorScheme, Mode, Theme},
    widget::{advanced::menu_overlay::PressProperties, navrail},
};
use iced_widget::{responsive, stack};
use log::{debug, error, trace};

use crate::{
    APP_NAME,
    gui::{
        dialog::Dialog,
        event_loop::{Event, Message},
        themes::THEMES,
    },
    settings::Settings,
};

pub(super) use crate::gui::widget::list::THUMBNAIL_SIZE;

#[derive(Default, Debug, Clone)]
pub enum LoadingState {
    #[default]
    Loading,
    Failed,
    Loaded,
}

pub struct Chilen {
    library: Option<Box<MusicLibrary>>,
    player_state: Option<PlayerState>,
    pub dialog: Dialog,
    loading_state: LoadingState,
    pub theme: Theme,
    pub settings: Settings,
    main_view: main_view::State,
    playback_view: playback_view::State,
    settings_state: settings::State,
    host_theme_preference: Mode,
    persistent: persistent_state::State,
}

impl Default for Chilen {
    fn default() -> Self {
        let mut state = Self {
            library: None,
            player_state: None,
            dialog: Dialog::Loading(None),
            loading_state: LoadingState::default(),
            theme: Theme::default(Mode::Light),
            settings: Settings::default(),
            main_view: main_view::State::default(),
            playback_view: playback_view::State::default(),
            settings_state: settings::State::default(),
            host_theme_preference: Mode::default(),
            persistent: persistent_state::State::load().unwrap_or_default(),
        };

        let settings = Settings::load().unwrap_or_default();
        state.apply_settings(settings);
        state
    }
}

static WINDOW_MODE: LazyLock<Arc<RwLock<Option<window::Mode>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

static EVENT_SENDER: LazyLock<Arc<RwLock<Option<mpsc::Sender<Event>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(None)));

const SPACING_SMALLER: f32 = 8.0;
const SPACING_SMALL: f32 = 12.0;
const SPACING_REGULAR: f32 = 16.0;
const SPACING_LARGE: f32 = 20.0;

const ROUNDING_SMALL: f32 = 14.0;
const ROUNDING_REGULAR: f32 = 14.0;
const ROUNDING_LARGE: f32 = 18.0;
const ROUNDING_LARGER: f32 = 20.0;
const GOTH_CORNER_ROUNDING: f32 = ROUNDING_LARGE;

const PLAYBACK_DESIRED_WIDTH: f32 = 410.0;
const MAIN_MIN_WIDTH: f32 = 450.0;
const PLAYBACK_MIN_WIDTH: f32 = 320.0;
const TOTAL_MIN_WIDTH: f32 =
    MAIN_MIN_WIDTH + PLAYBACK_MIN_WIDTH + navrail::CONTAINER_COLLAPSED_WIDTH;
// TODO: Should be calculated dynamically based on the window width
const MIN_HEIGHT: f32 = 920.0;

pub(super) fn event_sender_initialized() -> bool {
    EVENT_SENDER.read().unwrap().is_some()
}

pub(super) fn send_event(event: Event) {
    match EVENT_SENDER.write().unwrap().as_mut() {
        Some(sender) => {
            if let Err(e) = sender.try_send(event) {
                error!("Could not send the event: {e}");
            }
        }
        None => {
            error!("The sender is not initialized!")
        }
    }
}

pub fn menu_close<Message>(properties: PressProperties, message: Message) -> Option<Message> {
    properties
        .position
        .map(|pos| !properties.bounds.contains(pos))
        .unwrap_or_default()
        .then_some(message)
}

impl Chilen {
    pub fn save_persistent(&mut self) {
        debug!("Saving app state to disk");
        if let Err(e) = self.persistent.clone().save() {
            let msg = format!("Failed to save app state to disk: {e}");
            error!("{}", msg.clone());
            self.dialog = Dialog::Error(msg)
        }
    }

    fn apply_settings(&mut self, settings: Settings) {
        let theme = THEMES
            .iter()
            .find(|t| t.name == settings.theme_name)
            .cloned()
            .unwrap_or(THEMES[0].clone())
            .into_theme(settings.theme_mode);

        self.settings = settings;
        self.theme = theme;
    }

    fn view(state: &Chilen) -> Element<'_, Message> {
        let base_content = responsive(|size| {
            let compact = size.width
                <= MAIN_MIN_WIDTH + PLAYBACK_DESIRED_WIDTH + navrail::CONTAINER_EXPANDED_MIN_WIDTH;
            let playback_width = (size.width - MAIN_MIN_WIDTH - navrail::CONTAINER_COLLAPSED_WIDTH)
                .min(PLAYBACK_DESIRED_WIDTH);

            row![
                main_view::view(state, compact),
                playback_view::view(state, playback_width),
            ]
            .into()
        });

        let base = container(base_content)
            .style(|_| container::background(state.theme.surface_container()));

        stack![
            base,
            state
                .settings_state
                .screen
                .is_some()
                .then_some(settings::view(state)),
            dialog::view(state),
        ]
        .into()
    }
}

pub fn start() -> iced::Result {
    trace!("Launching GUI");
    let app = iced::application(Chilen::default, event_loop::update, Chilen::view)
        .font(icons::FILLED_ICONS_FONT_BYTES)
        .font(icons::OUTLINED_ICONS_FONT_BYTES)
        .font(font::BYTES_REGULAR)
        .font(font::BYTES_BOLD);

    #[cfg(feature = "cjk-fonts")]
    let app = app.font(font::BYTES_REGULAR_CJK).font(font::BYTES_BOLD_CJK);

    app.title(APP_NAME)
        .default_font(Font::with_name(font::NAME))
        .window(window::Settings {
            min_size: Some(iced::Size {
                width: TOTAL_MIN_WIDTH,
                height: MIN_HEIGHT,
            }),
            ..Default::default()
        })
        .subscription(|_| event_loop::subscription())
        .run()
}
