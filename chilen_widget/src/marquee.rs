use std::time::{Duration, Instant};

use iced::{
    Element, Length, Pixels, Point, Rectangle, Size, Vector,
    advanced::{
        Widget,
        layout::{Limits, Node},
        mouse,
        widget::{Tree, tree},
    },
};
use iced_m3::animation::Interpolable;

#[must_use]
pub fn marquee<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Marquee<'a, Message, Theme, Renderer> {
    Marquee::new(content)
}

/// The visual gap between the widget when scrolling.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum ScrollGap {
    /// Automatically detect the gap so that only one widget is visible at a time.
    ///
    /// Same as [`Relative(0.0)`](ScrollGap::Relative)
    #[default]
    Auto,
    /// Manual gap in pixels.
    Absolute(Pixels),
    /// Relative gap.
    ///
    /// Setting this to `0` means no gap, `1` is the same as [`Auto`](ScrollGap::Auto).
    Relative(f32),
}

/// Determines how the text is scrolled within a [`Marquee`].
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum Behavior {
    /// The text bounces back and forth between edges
    #[default]
    Alternate,

    /// The text continuously enters from the left and exits to the right.
    Scroll(ScrollGap),
}

impl Behavior {
    fn interpolate(
        self,
        content_width: f32,
        viewport_width: f32,
        elapsed: Duration,
        scroll_speed: ScrollSpeed,
        pause: Duration,
    ) -> (f32, Option<f32>) {
        let pause = pause.as_secs_f32();

        match self {
            Self::Alternate => {
                let to = (content_width - viewport_width).max(0.0);

                let travel = match scroll_speed {
                    ScrollSpeed::Relative(duration) => duration.as_secs_f32(),
                    ScrollSpeed::Absolute(pixels) => to / pixels.0,
                };

                if travel == 0.0 {
                    return (0.0, None);
                }

                let t = elapsed.as_secs_f32().rem_euclid(2.0 * (travel + pause));

                if t < travel {
                    (0.0.interpolate(to, t / travel), None)
                } else if t < travel + pause {
                    (to, None)
                } else if t < 2.0 * travel + pause {
                    (to.interpolate(0.0, (t - travel - pause) / travel), None)
                } else {
                    (0.0, None)
                }
            }
            Self::Scroll(gap) => {
                let gap = match gap {
                    ScrollGap::Auto => viewport_width,
                    ScrollGap::Absolute(pixels) => pixels.0.max(0.0),
                    ScrollGap::Relative(ratio) => ratio.max(0.0) * viewport_width,
                };

                let travel_distance = gap + content_width;
                let travel_time = match scroll_speed {
                    ScrollSpeed::Relative(duration) => duration.as_secs_f32(),
                    ScrollSpeed::Absolute(pixels) if pixels.0 > 0.0 => travel_distance / pixels.0,
                    ScrollSpeed::Absolute(_) => 0.0,
                };

                if travel_distance <= 0.0
                    || !travel_distance.is_finite()
                    || travel_time <= 0.0
                    || !travel_time.is_finite()
                {
                    return (0.0, None);
                }

                let t = elapsed.as_secs_f32().rem_euclid(travel_time + pause);

                if t < pause {
                    (0.0, None)
                } else {
                    let progress = (t - pause) / travel_time;

                    (
                        0.0.interpolate(-travel_distance, progress),
                        Some(travel_distance.interpolate(0.0, progress)),
                    )
                }
            }
        }
    }
}

/// The scroll speed of the [`Marquee`] content.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ScrollSpeed {
    /// Time required to complete one full loop of the content.
    Relative(Duration),
    /// Constant velocity in pixels per second.
    Absolute(Pixels),
}

impl Default for ScrollSpeed {
    fn default() -> Self {
        Self::Relative(Duration::from_secs(3))
    }
}

/// Specifies when the [`Marquee`] animation should play.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Animation {
    /// Never play the animation.
    Off,
    #[default]
    /// Only play the animation when the widget is hovered.
    WhenHovered,
    /// Always play the animation.
    Always,
}

// TODO: A fade effect for the widget around viewport edges would be nice
pub struct Marquee<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    pause: Duration,
    scroll_speed: ScrollSpeed,
    behavior: Behavior,
    animation: Animation,
}

impl<'a, Message, Theme, Renderer> Marquee<'a, Message, Theme, Renderer> {
    #[must_use]
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            pause: Duration::from_secs(1),
            scroll_speed: ScrollSpeed::default(),
            behavior: Behavior::default(),
            animation: Animation::default(),
        }
    }

    #[must_use]
    pub fn pause(mut self, pause: Duration) -> Self {
        self.pause = pause;
        self
    }

    #[must_use]
    pub fn scroll_speed(mut self, scroll_speed: ScrollSpeed) -> Self {
        self.scroll_speed = scroll_speed;
        self
    }

    #[must_use]
    pub fn behavior(mut self, behavior: Behavior) -> Self {
        self.behavior = behavior;
        self
    }

    #[must_use]
    pub fn animation(mut self, animation: Animation) -> Self {
        self.animation = animation;
        self
    }
}

struct State {
    content_overflow: bool,
    offsets: (f32, Option<f32>),
    is_over: bool,
    start: Instant,
    last_bounds: Rectangle,
    last_size: Size,
}

impl Default for State {
    fn default() -> Self {
        Self {
            content_overflow: false,
            offsets: (0.0, None),
            is_over: false,
            start: Instant::now(),
            last_bounds: Rectangle::new(Point::ORIGIN, Size::ZERO),
            last_size: Size::ZERO,
        }
    }
}

impl State {
    fn reset_offsets(&mut self, now: Instant) {
        self.offsets = (0.0, None);
        self.start = now;
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Marquee<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.content]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let min_bounds = limits.min();
        let max_bounds = limits.max();
        let node = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &Limits::new(min_bounds, Size::new(f32::INFINITY, max_bounds.height)),
        );

        let content_bounds = node.bounds();
        let state = tree.state.downcast_mut::<State>();
        state.content_overflow = content_bounds.width > max_bounds.width;

        let size = Size::new(
            content_bounds
                .width
                .min(max_bounds.width)
                .max(min_bounds.width),
            content_bounds
                .height
                .min(max_bounds.height)
                .max(min_bounds.height),
        );
        if state.last_bounds != content_bounds || state.last_size != size {
            state.reset_offsets(Instant::now());
            state.last_bounds = content_bounds;
            state.last_size = size;
        }

        Node::with_children(size, vec![node])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let (offset1, offset2) = state.offsets;

        // offset == 0    => no translation
        // offset == diff => widget is anchored to the right

        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.with_translation(Vector::new(-offset1, 0.0), |renderer| {
                let child_viewport = iced::Rectangle {
                    x: viewport.x + offset1,
                    y: viewport.y,
                    width: viewport.width,
                    height: viewport.height,
                };

                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout.child(0),
                    cursor,
                    &child_viewport,
                );
            });

            if let Some(offset) = offset2 {
                let child_viewport = iced::Rectangle {
                    x: viewport.x + offset,
                    y: viewport.y,
                    width: viewport.width,
                    height: viewport.height,
                };

                renderer.with_translation(Vector::new(-offset, 0.0), |renderer| {
                    self.content.as_widget().draw(
                        &tree.children[0],
                        renderer,
                        theme,
                        style,
                        layout.child(0),
                        cursor,
                        &child_viewport,
                    );
                });
            }
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.child(0),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<State>();
        if !state.content_overflow {
            return;
        }

        let is_over = cursor.is_over(layout.bounds());

        match event {
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_over != is_over && self.animation == Animation::WhenHovered {
                    state.reset_offsets(Instant::now());
                    state.is_over = is_over;
                }
            }
            iced::Event::Window(iced::window::Event::RedrawRequested(at)) => {
                let should_animate = state.content_overflow
                    && (self.animation == Animation::Always
                        || (self.animation == Animation::WhenHovered && is_over));

                let offsets = if should_animate {
                    self.behavior.interpolate(
                        state.last_bounds.width,
                        layout.bounds().width,
                        at.duration_since(state.start),
                        self.scroll_speed,
                        self.pause,
                    )
                } else {
                    (0.0, None)
                };

                if state.offsets != offsets {
                    state.offsets = offsets;
                    if !should_animate {
                        state.reset_offsets(*at);
                    }
                }
            }
            _ => {}
        }
        shell.request_redraw();
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: iced::advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        match cursor.is_over(layout.bounds()) {
            true => self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.child(0),
                cursor,
                viewport,
                renderer,
            ),
            false => mouse::Interaction::default(),
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Marquee<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + iced::advanced::text::Renderer,
{
    fn from(value: Marquee<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
