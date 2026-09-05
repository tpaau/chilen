use iced::{
    Border, Element, Length, Shadow,
    advanced::{Layout, Renderer, Widget, layout},
};

use crate::theme::ColorScheme;

const BAR_GAP: f32 = 4.0;
const STOP_INDICATOR_SIZE: f32 = 4.0;
const STOP_INDICATOR_PADDING: f32 = 2.0;
const BAR_HEIGHT: f32 = 8.0;

// TODO: Indeterminate mode (when progress is `None`)
// TODO: Wavy variant!!
pub struct ProgressBar<'a> {
    progress: f32,
    theme: &'a dyn ColorScheme,
    width: Length,
}

impl<'a> ProgressBar<'a> {
    #[must_use]
    pub fn new(progress: f32, theme: &'a impl ColorScheme) -> Self {
        Self {
            progress,
            theme,
            width: Length::Fill,
        }
    }
}

impl<'a, Message> Widget<Message, iced::Theme, iced::Renderer> for ProgressBar<'a> {
    fn size(&self) -> iced::Size<Length> {
        iced::Size {
            width: self.width,
            height: Length::Fixed(BAR_HEIGHT),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, BAR_HEIGHT)
    }

    fn draw(
        &self,
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let progress = self.progress.clamp(0.0, 1.0);

        let bar_width = ((bounds.width * (1.0 - progress)) - (BAR_GAP * progress)).max(0.0);
        let bar_height = bar_width.min(BAR_HEIGHT);
        renderer.fill_quad(
            iced::advanced::renderer::Quad {
                bounds: iced::Rectangle {
                    x: bounds.x + bounds.width - (bar_width - BAR_GAP * progress),
                    y: bounds.y + (BAR_HEIGHT - bar_height) * 0.5,
                    width: bar_width,
                    height: bar_height,
                },
                border: Border::default().rounded(bar_height / 2.0),
                shadow: Shadow::default(),
                snap: true,
            },
            self.theme.secondary_container(),
        );

        renderer.fill_quad(
            iced::advanced::renderer::Quad {
                bounds: iced::Rectangle {
                    x: bounds.x + bounds.width - STOP_INDICATOR_PADDING - STOP_INDICATOR_SIZE,
                    y: bounds.y + STOP_INDICATOR_PADDING,
                    width: STOP_INDICATOR_SIZE,
                    height: STOP_INDICATOR_SIZE,
                },
                border: Border::default().rounded(BAR_HEIGHT / 2.0),
                shadow: Shadow::default(),
                snap: true,
            },
            self.theme.primary(),
        );

        let bar_width = ((bounds.width * progress) - (BAR_GAP * (1.0 - progress))).max(0.0);
        let bar_height = bar_width.min(BAR_HEIGHT);
        renderer.fill_quad(
            iced::advanced::renderer::Quad {
                bounds: iced::Rectangle {
                    x: bounds.x,
                    y: bounds.y + (BAR_HEIGHT - bar_height) * 0.5,
                    width: bar_width,
                    height: bar_height,
                },
                border: Border::default().rounded(BAR_HEIGHT / 2.0),
                shadow: Shadow::default(),
                snap: true,
            },
            self.theme.primary(),
        );
    }
}

impl<'a, Message> From<ProgressBar<'a>> for Element<'a, Message> {
    fn from(progress: ProgressBar<'a>) -> Self {
        Element::new(progress)
    }
}
