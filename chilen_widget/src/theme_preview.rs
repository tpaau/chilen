use std::time::Instant;

use iced::{
    Border, Color, Element, Padding, Rectangle,
    advanced::{Widget, layout::atomic, renderer::Quad},
};
use iced_m3::animation::motion::{self, SpringValue, fast_effects, fast_spatial};

const DEFAULT_SIZE: f32 = 48.0;
const RING_PADDING: f32 = 2.0;
const RING_WIDTH: f32 = 2.0;
const OUTLINE_HOVER_OPACITY: f32 = 0.6;

/// Theme preview widget inspired by the one found in the Android color settings.
///
/// Displays a circle with three color sections, where the top one takes up half of the circle.
pub struct ThemePreview {
    color_top: Color,
    color_left: Color,
    color_right: Color,
    size: f32,
    is_hovered: bool,
    selected: bool,
    motion_scheme: Option<motion::Scheme>,
}

impl ThemePreview {
    #[must_use]
    pub fn new(color_top: Color, color_left: Color, color_right: Color) -> Self {
        Self {
            color_top,
            color_left,
            color_right,
            size: DEFAULT_SIZE,
            is_hovered: false,
            selected: false,
            motion_scheme: None,
        }
    }

    #[must_use]
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[must_use]
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    pub fn motion_scheme(mut self, scheme: motion::Scheme) -> Self {
        self.motion_scheme = Some(scheme);
        self
    }
}

struct State {
    corner_radius_spring: SpringValue<f32>,
    outline_opacity_spring: SpringValue<f32>,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ThemePreview
where
    Renderer: iced::advanced::Renderer,
{
    fn state(&self) -> iced::advanced::widget::tree::State {
        let now = Instant::now();
        iced::advanced::widget::tree::State::new(State {
            corner_radius_spring: SpringValue::new(
                match self.selected {
                    true => self.size / 3.0,
                    false => self.size / 2.0,
                },
                fast_spatial(self.motion_scheme.unwrap_or_default()),
                now,
            ),
            outline_opacity_spring: SpringValue::new(
                self.selected as usize as f32,
                fast_effects(self.motion_scheme.unwrap_or_default()),
                now,
            ),
        })
    }

    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn size(&self) -> iced::Size<iced::Length> {
        iced::Size {
            width: iced::Length::Fixed(self.size),
            height: iced::Length::Fixed(self.size),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        atomic(limits, self.size, self.size)
    }

    fn draw(
        &self,
        tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let radius = state.corner_radius_spring.value();
        let full_bounds = layout.bounds();
        let padding = Padding::from(RING_PADDING + RING_WIDTH);
        let bounds = full_bounds.shrink(padding);

        let draw_part = |renderer: &mut Renderer, clip: iced::Rectangle, color: Color| {
            if let Some(clip) = clip.intersection(viewport) {
                renderer.with_layer(clip, |renderer| {
                    renderer.fill_quad(
                        Quad {
                            bounds,
                            border: Border::default().rounded(radius),
                            ..Default::default()
                        },
                        color,
                    );
                });
            }
        };

        draw_part(
            renderer,
            Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height / 2.0,
            },
            self.color_top,
        );

        draw_part(
            renderer,
            Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height / 2.0,
                width: bounds.width / 2.0,
                height: bounds.height / 2.0,
            },
            self.color_left,
        );

        draw_part(
            renderer,
            Rectangle {
                x: bounds.x + bounds.width / 2.0,
                y: bounds.y + bounds.height / 2.0,
                width: bounds.width / 2.0,
                height: bounds.height / 2.0,
            },
            self.color_right,
        );

        renderer.fill_quad(
            Quad {
                bounds: full_bounds,
                border: Border::default()
                    .rounded(radius + RING_PADDING + RING_WIDTH)
                    .color(
                        self.color_top
                            .scale_alpha(state.outline_opacity_spring.value()),
                    )
                    .width(RING_WIDTH),
                ..Default::default()
            },
            Color::TRANSPARENT,
        );
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        _event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        self.is_hovered = cursor.is_over(layout.bounds());

        let now = Instant::now();
        let state = tree.state.downcast_mut::<State>();

        let radius = match self.selected {
            true => self.size / 3.0,
            false => self.size / 2.0,
        };
        state.corner_radius_spring.set_target(radius, now);

        let opacity = if self.selected {
            1.0
        } else if self.is_hovered {
            OUTLINE_HOVER_OPACITY
        } else {
            0.0
        };
        state.outline_opacity_spring.set_target(opacity, now);

        if !state.corner_radius_spring.is_at_rest() || !state.outline_opacity_spring.is_at_rest() {
            shell.request_redraw();
        }

        state.corner_radius_spring.step(now);
        state.outline_opacity_spring.step(now);
    }
}

impl<'a, Message> From<ThemePreview> for Element<'a, Message> {
    fn from(value: ThemePreview) -> Self {
        Element::new(value)
    }
}
