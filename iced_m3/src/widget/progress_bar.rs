use std::time::Instant;

use iced::{
    Border, Color, Element, Length, Shadow,
    advanced::{Layout, Renderer, Widget, layout, widget::tree},
};

use crate::{
    animation::{EMPHASIZED_ACCELERATE, cubic_bezier_tuple},
    theme::ColorScheme,
};

const STOP_INDICATOR_SIZE: f32 = 4.0;
const MIN_HEIGHT: f32 = 8.0;

#[derive(Debug)]
struct State {
    start: Option<Instant>,
}

impl State {
    fn new() -> Self {
        Self { start: None }
    }
}

// TODO: Wavy variant!!
pub struct ProgressBar<'a> {
    progress: Option<f32>,
    height: Option<f32>,
    theme: &'a dyn ColorScheme,
    width: Length,
}

impl<'a> ProgressBar<'a> {
    #[must_use]
    pub fn new(theme: &'a impl ColorScheme) -> Self {
        Self {
            progress: None,
            height: None,
            theme,
            width: Length::Fill,
        }
    }

    #[must_use]
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress);
        self
    }

    #[must_use]
    pub fn progress_maybe(mut self, maybe_progress: Option<f32>) -> Self {
        self.progress = maybe_progress;
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

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for ProgressBar<'a> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        tree::State::new(State::new())
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size {
            width: self.width,
            height: Length::Fixed(self.height.unwrap_or(MIN_HEIGHT).max(MIN_HEIGHT)),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(
            limits,
            self.width,
            self.height.unwrap_or(MIN_HEIGHT).max(MIN_HEIGHT),
        )
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        _event: &iced::Event,
        _layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        if self.progress.is_none() {
            if state.start.is_none() {
                state.start = Some(Instant::now());
            }
            shell.request_redraw();
        } else {
            state.start = None;
        }
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let target_height = self.height.unwrap_or(MIN_HEIGHT).max(MIN_HEIGHT);
        let gap = target_height / 2.0;

        let mut draw_quad = |x: f32, width: f32, height: f32, color: Color| {
            if width <= 0.0 {
                return;
            }

            renderer.fill_quad(
                iced::advanced::renderer::Quad {
                    bounds: iced::Rectangle {
                        x,
                        y: bounds.y + (target_height - height) / 2.0,
                        width,
                        height,
                    },
                    border: Border::default().rounded(height / 2.0),
                    shadow: Shadow::default(),
                    snap: true,
                },
                color,
            );
        };

        match self.progress {
            Some(progress) => {
                let bar_width = ((bounds.width * (1.0 - progress)) - (gap * progress)).max(0.0);
                let bar_height = bar_width.min(target_height);
                draw_quad(
                    bounds.x + bounds.width - bar_width,
                    bar_width,
                    bar_height,
                    self.theme.secondary_container(),
                );

                draw_quad(
                    bounds.x + bounds.width
                        - STOP_INDICATOR_SIZE
                        - (target_height - STOP_INDICATOR_SIZE) / 2.0,
                    STOP_INDICATOR_SIZE,
                    STOP_INDICATOR_SIZE,
                    self.theme.primary(),
                );

                let bar_width = ((bounds.width * progress) - (gap * (1.0 - progress))).max(0.0);
                let bar_height = bar_width.min(target_height);
                draw_quad(bounds.x, bar_width, bar_height, self.theme.primary());
            }
            None => {
                let state = tree.state.downcast_ref::<State>();
                // Update always runs before draw so it's guaranteed to be Some?
                let elapsed = state.start.unwrap().elapsed().as_secs_f32();

                const CYCLE: f32 = 1.75;
                let time = elapsed % CYCLE;

                let ease = |t: f32| cubic_bezier_tuple(t, EMPHASIZED_ACCELERATE);

                let animation_value = |delay: f32, duration: f32| -> f32 {
                    let t = ((time - delay) / duration).clamp(0.0, 1.0);
                    ease(t)
                };

                let first_head = animation_value(0.0, 1.0);
                let first_tail = animation_value(0.25, 1.0);

                let second_head = animation_value(0.65, 0.85);
                let second_tail = animation_value(0.90, 0.85);

                let gap = gap / bounds.width.max(1.0);

                // Convert the animated positions into intervals.
                let mut primary_segments = [
                    (first_tail.min(first_head), first_tail.max(first_head)),
                    (second_tail.min(second_head), second_tail.max(second_head)),
                ];

                // The two animated segments can cross during the animation.
                // Sort them so the track pieces are always drawn left-to-right.
                primary_segments.sort_by(|a, b| a.0.total_cmp(&b.0));

                let [(first_start, first_end), (second_start, second_end)] = primary_segments;

                let mut draw_normalized = |start: f32, end: f32, color: Color| {
                    let start = start.clamp(0.0, 1.0);
                    let end = end.clamp(0.0, 1.0);

                    if end <= start {
                        return;
                    }

                    draw_quad(
                        bounds.x + bounds.width * start,
                        bounds.width * (end - start),
                        target_height,
                        color,
                    );
                };

                // Track segment 1.
                draw_normalized(
                    0.0,
                    first_start - gap / 2.0,
                    self.theme.secondary_container(),
                );

                // Track segment 2.
                draw_normalized(
                    first_end + gap / 2.0,
                    second_start - gap / 2.0,
                    self.theme.secondary_container(),
                );

                // Track segment 3.
                draw_normalized(
                    second_end + gap / 2.0,
                    1.0,
                    self.theme.secondary_container(),
                );

                // Primary segment 1.
                draw_normalized(
                    first_start + gap / 2.0,
                    first_end - gap / 2.0,
                    self.theme.primary(),
                );

                // Primary segment 2.
                draw_normalized(
                    second_start + gap / 2.0,
                    second_end - gap / 2.0,
                    self.theme.primary(),
                );
            }
        }
    }
}

impl<'a, Message> From<ProgressBar<'a>> for Element<'a, Message> {
    fn from(progress: ProgressBar<'a>) -> Self {
        Element::new(progress)
    }
}
