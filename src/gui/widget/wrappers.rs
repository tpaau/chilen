use std::time::Duration;

use chilen_widget::marquee::{Behavior, Marquee};
use iced::Element;

#[must_use]
pub fn marquee<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Marquee<'a, Message, Theme, Renderer> {
    Marquee::new(content)
        .pause(Duration::from_secs(2))
        .scroll_time(Duration::from_secs(6))
        .behavior(Behavior::Scroll)
}
