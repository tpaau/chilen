use iced::{
    Border, Color, Element, Padding, Rectangle,
    advanced::{Widget, layout::atomic, renderer::Quad},
};

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
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for ThemePreview
where
    Renderer: iced::advanced::Renderer,
{
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
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let full_bounds = layout.bounds();
        let padding = Padding::from(RING_PADDING + RING_WIDTH);
        let bounds = full_bounds.shrink(padding);
        let radius = match self.selected {
            true => bounds.width / 3.0,
            false => f32::MAX,
        };

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

        if self.selected {
            renderer.fill_quad(
                Quad {
                    bounds: full_bounds,
                    border: Border::default()
                        .rounded(radius + RING_PADDING + RING_WIDTH)
                        .color(self.color_top)
                        .width(RING_WIDTH),
                    ..Default::default()
                },
                Color::TRANSPARENT,
            );
        } else if self.is_hovered {
            renderer.fill_quad(
                Quad {
                    bounds: full_bounds,
                    border: Border::default()
                        .rounded(f32::MAX)
                        .color(self.color_top.scale_alpha(OUTLINE_HOVER_OPACITY))
                        .width(RING_WIDTH),
                    ..Default::default()
                },
                Color::TRANSPARENT,
            );
        }
    }

    fn update(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        _shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        self.is_hovered = cursor.is_over(layout.bounds());
    }
}

impl<'a, Message> From<ThemePreview> for Element<'a, Message> {
    fn from(value: ThemePreview) -> Self {
        Element::new(value)
    }
}
