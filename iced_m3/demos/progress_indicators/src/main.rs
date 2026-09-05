use std::time::{Duration, Instant};

use iced::{
    Element, Font, Length, Subscription, Task,
    widget::{column, container, text},
};
use iced_m3::{
    theme::{ColorScheme, Theme},
    widget::progress_bar,
};

const APP_NAME: &str = "Progress Indicators Demo";

struct State {
    start: Instant,
    progress: f32,
    theme: Theme,
}

impl Default for State {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            progress: 0.0,
            theme: Theme::default(iced_m3::theme::Mode::Dark),
        }
    }
}

pub fn anim(elapsed: Duration) -> f32 {
    const RAMP: f32 = 1.2;
    const HOLD: f32 = 0.75;
    const RESET: f32 = 0.05;
    const WAIT: f32 = 0.15;

    let cycle = RAMP + HOLD + RESET + WAIT;
    let t = elapsed.as_secs_f32() % cycle;

    if t < RAMP {
        let progress = t / RAMP;
        1.0 - (1.0 - progress).powi(2)
    } else if t < RAMP + HOLD {
        1.0
    } else {
        0.0
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.progress = anim(self.start.elapsed());
            }
        }
        Task::none()
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        container(
            column![
                text(APP_NAME)
                    .font(fonts::text_bold())
                    .size(24.0)
                    .color(self.theme.on_surface()),
                text("Determinate indicators").color(self.theme.on_surface()),
                progress_bar(&self.theme).progress(0.0),
                progress_bar(&self.theme).progress(0.25),
                progress_bar(&self.theme).progress(0.5),
                progress_bar(&self.theme).progress(0.75),
                progress_bar(&self.theme).progress(1.0),
                text("You can change height (thick)").color(self.theme.on_surface()),
                progress_bar(&self.theme).progress(0.5).height(16.0),
                text("In action!").color(self.theme.on_surface()),
                progress_bar(&self.theme).progress(self.progress),
                progress_bar(&self.theme)
                    .progress(self.progress)
                    .height(16.0),
                text("Indeterminate!").color(self.theme.on_surface()),
                progress_bar(&self.theme),
                progress_bar(&self.theme).height(16.0),
            ]
            .width(500.0)
            .spacing(20.0),
        )
        .padding(10.0)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| iced::widget::container::Style::default().background(self.theme.background()))
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
    }
}

enum Message {
    Tick,
}

fn main() -> iced::Result {
    iced::application(
        || {
            (
                State::default(),
                Task::batch(
                    fonts::get_fonts()
                        .into_iter()
                        .map(|b| iced::font::load(b).discard()),
                ),
            )
        },
        State::update,
        State::view,
    )
    .subscription(State::subscription)
    .title(APP_NAME)
    .default_font(Font::with_name("Noto Sans"))
    .run()
}
