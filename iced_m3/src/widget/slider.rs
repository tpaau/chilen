use std::ops::RangeInclusive;

use iced::{
    Border, Element, Event, Length, Point, Rectangle,
    advanced::{Widget, layout, mouse},
    border::Radius,
    keyboard::{self, Key, key},
    touch, window,
};
use iced_widget::core::renderer;

use crate::theme::ColorScheme;

const HANDLE_WIDTH_IDLE: f32 = 4.0;
const HANDLE_WIDTH_DRAGGED: f32 = 2.0;
const HANDLE_GAP: f32 = 6.0;
const INNER_CORNER_RADIUS: f32 = 2.0;
const STOP_INDICATOR_SIZE: f32 = 4.0;
const STOP_INDICATOR_TRAILING_SPACE: f32 = 4.0;

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum Size {
    #[default]
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    Custom {
        rail_height: f32,
        handle_height: f32,
        corner_radius: f32,
    },
}

impl Size {
    pub fn rail_height(&self) -> f32 {
        match self {
            Size::ExtraSmall => 16.0,
            Size::Small => 24.0,
            Size::Medium => 40.0,
            Size::Large => 56.0,
            Size::ExtraLarge => 96.0,
            Size::Custom {
                rail_height,
                handle_height: _,
                corner_radius: _,
            } => *rail_height,
        }
    }

    pub fn handle_height(&self) -> f32 {
        match self {
            Size::ExtraSmall | Size::Small => 44.0,
            Size::Medium => 52.0,
            Size::Large => 68.0,
            Size::ExtraLarge => 108.0,
            Size::Custom {
                rail_height: _,
                handle_height,
                corner_radius: _,
            } => *handle_height,
        }
    }

    pub fn corner_radius(&self) -> f32 {
        match self {
            Size::ExtraSmall | Size::Small => 8.0,
            Size::Medium => 12.0,
            Size::Large => 16.0,
            Size::ExtraLarge => 28.0,
            Size::Custom {
                rail_height: _,
                handle_height: _,
                corner_radius,
            } => *corner_radius,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_dragging: bool,
    keyboard_modifiers: iced::keyboard::Modifiers,
}

/// The possible status of a [`Slider`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    /// The [`Slider`] can be interacted with.
    Active,
    /// The [`Slider`] is being hovered.
    Hovered,
    /// The [`Slider`] is being dragged.
    Dragged,
}

pub struct Slider<'a, T, Message> {
    range: RangeInclusive<T>,
    step: T,
    shift_step: Option<T>,
    value: T,
    default: Option<T>,
    on_change: Box<dyn Fn(T) -> Message + 'a>,
    on_release: Option<Message>,
    width: Length,
    theme: &'a dyn ColorScheme,
    size: Size,
    status: Option<Status>,
}

impl<'a, T, Message> Slider<'a, T, Message>
where
    T: Copy + From<u8> + PartialOrd,
    Message: Clone,
{
    pub fn new<F>(
        range: RangeInclusive<T>,
        value: T,
        on_change: F,
        theme: &'a impl ColorScheme,
    ) -> Self
    where
        F: 'a + Fn(T) -> Message,
    {
        let value = if value >= *range.start() {
            value
        } else {
            *range.start()
        };

        let value = if value <= *range.end() {
            value
        } else {
            *range.end()
        };

        Slider {
            value,
            default: None,
            range,
            step: T::from(1),
            shift_step: None,
            on_change: Box::new(on_change),
            on_release: None,
            width: Length::Fill,
            theme,
            size: Size::default(),
            status: None,
        }
    }

    /// Sets the optional default value for the [`Slider`].
    ///
    /// If set, the [`Slider`] will reset to this value when ctrl-clicked or command-clicked.
    #[must_use]
    pub fn default(mut self, default: impl Into<T>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Sets the release message of the [`Slider`].
    /// This is called when the mouse is released from the slider.
    ///
    /// Typically, the user's interaction with the slider is finished when this message is produced.
    /// This is useful if you need to spawn a long-running task from the slider's result, where
    /// the default on_change message could create too many events.
    #[must_use]
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets the width of the [`Slider`].
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the step size of the [`Slider`].
    #[must_use]
    pub fn step(mut self, step: impl Into<T>) -> Self {
        self.step = step.into();
        self
    }

    /// Sets the optional "shift" step for the [`Slider`].
    ///
    /// If set, this value is used as the step while the shift key is pressed.
    #[must_use]
    pub fn shift_step(mut self, shift_step: impl Into<T>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    #[must_use]
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Slider<'_, T, Message>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive,
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        iced::advanced::widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> iced::advanced::widget::tree::State {
        iced::advanced::widget::tree::State::new(State::default())
    }

    fn size(&self) -> iced::Size<Length> {
        iced::Size {
            width: self.width,
            height: iced::Length::Fixed(self.size.handle_height()),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut iced::advanced::widget::Tree,
        _renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.size.handle_height())
    }

    fn update(
        &mut self,
        tree: &mut iced::advanced::widget::Tree,
        event: &iced::Event,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let mut update = || {
            let current_value = self.value;

            let locate = |cursor_position: Point| -> Option<T> {
                let bounds = layout.bounds();

                if cursor_position.x <= bounds.x {
                    Some(*self.range.start())
                } else if cursor_position.x >= bounds.x + bounds.width {
                    Some(*self.range.end())
                } else {
                    let step = if state.keyboard_modifiers.shift() {
                        self.shift_step.unwrap_or(self.step)
                    } else {
                        self.step
                    }
                    .into();

                    let start = (*self.range.start()).into();
                    let end = (*self.range.end()).into();

                    let percent = f64::from(cursor_position.x - bounds.x) / f64::from(bounds.width);

                    let steps = (percent * (end - start) / step).round();
                    let value = steps * step + start;

                    T::from_f64(value.min(end))
                }
            };

            let increment = |value: T| -> Option<T> {
                let step = if state.keyboard_modifiers.shift() {
                    self.shift_step.unwrap_or(self.step)
                } else {
                    self.step
                }
                .into();

                let steps = (value.into() / step).round();
                let new_value = step * (steps + 1.0);

                if new_value > (*self.range.end()).into() {
                    return Some(*self.range.end());
                }

                T::from_f64(new_value)
            };

            let decrement = |value: T| -> Option<T> {
                let step = if state.keyboard_modifiers.shift() {
                    self.shift_step.unwrap_or(self.step)
                } else {
                    self.step
                }
                .into();

                let steps = (value.into() / step).round();
                let new_value = step * (steps - 1.0);

                if new_value < (*self.range.start()).into() {
                    return Some(*self.range.start());
                }

                T::from_f64(new_value)
            };

            let change = |new_value: T| {
                if (self.value.into() - new_value.into()).abs() > f64::EPSILON {
                    shell.publish((self.on_change)(new_value));

                    self.value = new_value;
                }
            };

            match &event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(cursor_position) = cursor.position_over(layout.bounds()) {
                        if state.keyboard_modifiers.command() {
                            let _ = self.default.map(change);
                            state.is_dragging = false;
                        } else {
                            let _ = locate(cursor_position).map(change);
                            state.is_dragging = true;
                        }

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    if state.is_dragging {
                        if let Some(on_release) = self.on_release.clone() {
                            shell.publish(on_release);
                        }
                        state.is_dragging = false;
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    if state.is_dragging {
                        let _ = cursor.land().position().and_then(locate).map(change);

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::WheelScrolled { delta })
                    if state.keyboard_modifiers.control() =>
                {
                    if cursor.is_over(layout.bounds()) {
                        let delta = match delta {
                            mouse::ScrollDelta::Lines { x: _, y } => y,
                            mouse::ScrollDelta::Pixels { x: _, y } => y,
                        };

                        if *delta < 0.0 {
                            let _ = decrement(current_value).map(change);
                        } else {
                            let _ = increment(current_value).map(change);
                        }

                        shell.capture_event();
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    if cursor.is_over(layout.bounds()) {
                        match key {
                            Key::Named(key::Named::ArrowUp) => {
                                let _ = increment(current_value).map(change);
                                shell.capture_event();
                            }
                            Key::Named(key::Named::ArrowDown) => {
                                let _ = decrement(current_value).map(change);
                                shell.capture_event();
                            }
                            _ => (),
                        }
                    }
                }
                Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                    state.keyboard_modifiers = *modifiers;
                }
                _ => {}
            }
        };

        update();

        let current_status = if state.is_dragging {
            Status::Dragged
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.status = Some(current_status);
        } else if self.status.is_some_and(|status| status != current_status) {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        _tree: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: iced::advanced::Layout<'_>,
        _cursor: iced::advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let status = self.status.unwrap();

        let value = self.value.into() as f32;
        let (range_start, range_end) = {
            let (start, end) = self.range.clone().into_inner();
            (start.into() as f32, end.into() as f32)
        };

        let offset = if range_start >= range_end {
            0.0
        } else {
            (bounds.width - HANDLE_WIDTH_IDLE) * (value - range_start) / (range_end - range_start)
        };

        let rail_y = bounds.y + bounds.height / 2.0;
        let rail_height = self.size.rail_height();

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + offset + HANDLE_WIDTH_IDLE / 2.0 + HANDLE_GAP,
                    y: rail_y - rail_height / 2.0,
                    width: bounds.width - offset - HANDLE_WIDTH_IDLE / 2.0 - HANDLE_GAP,
                    height: rail_height,
                },
                border: Border::default().rounded(Radius {
                    top_left: INNER_CORNER_RADIUS,
                    top_right: self.size.corner_radius(),
                    bottom_right: self.size.corner_radius(),
                    bottom_left: INNER_CORNER_RADIUS,
                }),
                ..renderer::Quad::default()
            },
            self.theme.secondary_container(),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y - rail_height / 2.0,
                    width: offset - HANDLE_GAP - HANDLE_WIDTH_IDLE / 2.0,
                    height: rail_height,
                },
                border: Border::default().rounded(Radius {
                    top_left: self.size.corner_radius(),
                    top_right: INNER_CORNER_RADIUS,
                    bottom_right: INNER_CORNER_RADIUS,
                    bottom_left: self.size.corner_radius(),
                }),
                ..renderer::Quad::default()
            },
            self.theme.primary(),
        );

        let handle_width = match status {
            Status::Active | Status::Hovered => HANDLE_WIDTH_IDLE,
            Status::Dragged => HANDLE_WIDTH_DRAGGED,
        };
        let handle_x = bounds.x + offset - handle_width / 2.0;
        let stop_indicator_width = (bounds.x + bounds.width
            - STOP_INDICATOR_TRAILING_SPACE
            - STOP_INDICATOR_SIZE
            - handle_x
            - HANDLE_GAP
            + 1.0)
            .clamp(0.0, STOP_INDICATOR_SIZE);
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + bounds.width
                        - STOP_INDICATOR_TRAILING_SPACE
                        - stop_indicator_width,
                    y: rail_y - STOP_INDICATOR_SIZE / 2.0,
                    width: stop_indicator_width,
                    height: STOP_INDICATOR_SIZE,
                },
                border: Border::default().rounded(f32::MAX),
                ..renderer::Quad::default()
            },
            self.theme.primary(),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: handle_x,
                    y: rail_y - self.size.handle_height() / 2.0,
                    width: handle_width,
                    height: self.size.handle_height(),
                },
                border: Border::default().rounded(f32::MAX),
                ..renderer::Quad::default()
            },
            self.theme.primary(),
        );
    }

    fn mouse_interaction(
        &self,
        tree: &iced::advanced::widget::Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();

        if state.is_dragging {
            // FIXME: Fall back to `Pointer` on Windows
            // See https://github.com/rust-windowing/winit/issues/1043
            if cfg!(target_os = "windows") {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grabbing
            }
        } else if cursor.is_over(layout.bounds()) {
            if cfg!(target_os = "windows") {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::Grab
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, T, Message, Renderer> From<Slider<'a, T, Message>> for Element<'a, Message, Renderer>
where
    T: Copy + Into<f64> + num_traits::FromPrimitive + 'a,
    Message: Clone + 'a,
    Renderer: 'a,
{
    fn from(slider: Slider<'a, T, Message>) -> Element<'a, Message, Renderer> {
        Element::new(slider)
    }
}
