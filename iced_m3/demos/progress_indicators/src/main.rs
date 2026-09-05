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

pub fn oscillating_value(elapsed: Duration, motion: Duration, pause: Duration) -> f32 {
    let motion_secs = motion.as_secs_f32();
    let pause_secs = pause.as_secs_f32();

    if motion_secs <= 0.0 {
        return 0.0;
    }

    let cycle = 2.0 * (motion_secs + pause_secs);
    let t = elapsed.as_secs_f32() % cycle;

    let pi = std::f32::consts::PI;

    if t < pause_secs {
        0.0
    } else if t < pause_secs + motion_secs {
        let x = (t - pause_secs) / motion_secs;
        0.5 - 0.5 * (pi * x).cos()
    } else if t < pause_secs + motion_secs + pause_secs {
        1.0
    } else {
        let x = (t - pause_secs - motion_secs - pause_secs) / motion_secs;
        0.5 + 0.5 * (pi * x).cos()
    }
}

impl State {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.progress = oscillating_value(
                    self.start.elapsed(),
                    Duration::from_secs_f32(1.5),
                    Duration::from_secs_f32(0.6),
                );
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
                progress_bar(0.0, &self.theme),
                progress_bar(0.25, &self.theme),
                progress_bar(0.5, &self.theme),
                progress_bar(0.75, &self.theme),
                progress_bar(1.0, &self.theme),
                progress_bar(self.progress, &self.theme),
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
