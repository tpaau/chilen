use iced::{
    Border, Color, Element, Rectangle,
    advanced::{Widget, layout::atomic, renderer::Quad},
};

/// AOSP theme preview widget. Displays a circle with three color sections.
pub struct ThemePreview {
    pub color_top: Color,
    pub color_left: Color,
    pub color_right: Color,
    pub size: f32,
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
        let bounds = layout.bounds();

        let draw_part = |renderer: &mut Renderer, clip: iced::Rectangle, color: Color| {
            if let Some(clip) = clip.intersection(viewport) {
                renderer.with_layer(clip, |renderer| {
                    renderer.fill_quad(
                        Quad {
                            bounds,
                            border: Border::default().rounded(f32::MAX),
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
    }
}

impl<'a, Message> From<ThemePreview> for Element<'a, Message> {
    fn from(value: ThemePreview) -> Self {
        Element::new(value)
    }
}
