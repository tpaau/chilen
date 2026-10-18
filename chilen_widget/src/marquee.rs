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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Determines how the text is scrolled within a [`Marquee`].
pub enum Behavior {
    /// The text bounces back and forth between edges
    #[default]
    Alternate,

    // TODO: Maybe this one should draw the widget twice so it's always visible?
    /// The text continuously enters from the left and exits to the right.
    Scroll,
}

impl Behavior {
    fn interpolate(
        self,
        content_width: f32,
        viewport_width: f32,
        elapsed: Duration,
        scroll_speed: ScrollSpeed,
        pause: Duration,
    ) -> f32 {
        let pause = pause.as_secs_f32();

        match self {
            Self::Alternate => {
                let to = (content_width - viewport_width).max(0.0);

                let travel = match scroll_speed {
                    ScrollSpeed::Relative(duration) => duration.as_secs_f32(),
                    ScrollSpeed::Absolute(pixels) => to / pixels.0,
                };

                if travel == 0.0 {
                    return 0.0;
                }

                let t = elapsed.as_secs_f32().rem_euclid(2.0 * (travel + pause));

                if t < travel {
                    0.0.interpolate(to, t / travel)
                } else if t < travel + pause {
                    to
                } else if t < 2.0 * travel + pause {
                    to.interpolate(0.0, (t - travel - pause) / travel)
                } else {
                    0.0
                }
            }
            Self::Scroll => {
                let total_distance = content_width + viewport_width;
                let travel = match scroll_speed {
                    ScrollSpeed::Relative(duration) => duration.as_secs_f32(),
                    ScrollSpeed::Absolute(pixels) => total_distance / pixels.0,
                };

                if travel == 0.0 {
                    return 0.0;
                }

                let left_travel = travel * content_width / total_distance;
                let right_travel = travel - left_travel;
                let t = elapsed.as_secs_f32().rem_euclid(travel + pause);

                if t < pause {
                    0.0
                } else if t < right_travel + pause {
                    0.0.interpolate(-viewport_width, (t - pause) / right_travel)
                } else {
                    content_width.interpolate(0.0, (t - pause - right_travel) / left_travel)
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

// TODO: A fade effect for the widget around viewport edges would be nice
pub struct Marquee<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    pause: Duration,
    scroll_speed: ScrollSpeed,
    behavior: Behavior,
}

impl<'a, Message, Theme, Renderer> Marquee<'a, Message, Theme, Renderer> {
    #[must_use]
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            pause: Duration::from_secs(1),
            scroll_speed: ScrollSpeed::default(),
            behavior: Behavior::default(),
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
}

struct State {
    animate: bool,
    offset: f32,
    start: Instant,
    last_bounds: Rectangle,
    last_size: Size,
}

impl Default for State {
    fn default() -> Self {
        Self {
            animate: false,
            offset: 0.0,
            start: Instant::now(),
            last_bounds: Rectangle::new(Point::ORIGIN, Size::ZERO),
            last_size: Size::ZERO,
        }
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
        let state = tree.state.downcast_mut::<State>();
        state.offset = 0.0;
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

        let bounds = node.bounds();
        let state = tree.state.downcast_mut::<State>();
        state.animate = bounds.width - max_bounds.width < 0.0;

        let size = Size::new(
            bounds.width.min(max_bounds.width).max(min_bounds.width),
            bounds.height.min(max_bounds.height).max(min_bounds.height),
        );
        if state.last_bounds != bounds || state.last_size != size {
            state.start = Instant::now();
            state.offset = 0.0;
            state.last_bounds = bounds;
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

        let child_viewport = iced::Rectangle {
            x: viewport.x + state.offset,
            y: viewport.y,
            width: viewport.width,
            height: viewport.height,
        };

        // offset == 0    => no translation
        // offset == diff => widget is anchored to the right

        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.with_translation(Vector::new(-state.offset, 0.0), |renderer| {
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
        if let iced::Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_mut::<State>();
            let offset = match state.animate {
                true => 0.0,
                false => self.behavior.interpolate(
                    state.last_bounds.width,
                    layout.bounds().width,
                    now.duration_since(state.start),
                    self.scroll_speed,
                    self.pause,
                ),
            };
            if offset != state.offset {
                state.offset = offset;
                shell.request_redraw();
            }
        }

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
