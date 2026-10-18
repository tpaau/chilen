use std::time::Duration;

use chilen_widget::marquee::{Behavior, Marquee, ScrollGap, ScrollSpeed};
use iced::{Element, Pixels};

#[must_use]
pub fn marquee<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Marquee<'a, Message, Theme, Renderer> {
    Marquee::new(content)
        .pause(Duration::from_secs(3))
        .scroll_speed(ScrollSpeed::Absolute(Pixels(150.0)))
        .behavior(Behavior::Scroll(ScrollGap::Relative(0.5)))
}
