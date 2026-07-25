use std::{cell::Cell, rc::Rc};

use iced::advanced::Clipboard;
use iced_widget::core::{
    Element, Event, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget, keyboard,
    layout::{Limits, Node},
    mouse::{self, Cursor, Interaction},
    overlay,
    renderer::Style,
    widget::{Operation, Tree, tree},
};

#[derive(Debug)]
struct State {
    position: Option<Point>,
}

/// Describes which edge of `content` the menu is anchored to, and where along that edge it is placed.
///
/// Variant names are `EdgePosition`:
/// - `Edge` is the side of `content` the menu attaches to
/// - `Position` is the menu's alignment along that edge
///
/// For example, [`TopLeft`](Placement::TopLeft) means the menu is attached to the top edge of
/// `content` and aligned to the left. [`LeftTop`](Placement::LeftTop) means it is attached to the
/// left edge and aligned to the top.
#[derive(Default, Clone, Copy)]
pub enum Placement {
    TopLeft,
    TopCenter,
    TopRight,
    RightTop,
    RightCenter,
    RightBottom,
    BottomRight,
    BottomCenter,
    #[default]
    BottomLeft,
    LeftBottom,
    LeftCenter,
    LeftTop,
}

impl Placement {
    fn flip_x(&mut self) {
        *self = match self {
            Self::TopLeft => Self::TopRight,
            Self::TopRight => Self::TopLeft,
            Self::RightTop => Self::LeftTop,
            Self::RightCenter => Self::LeftCenter,
            Self::RightBottom => Self::LeftBottom,
            Self::BottomRight => Self::BottomLeft,
            Self::BottomLeft => Self::BottomRight,
            Self::LeftBottom => Self::RightBottom,
            Self::LeftCenter => Self::RightCenter,
            Self::LeftTop => Self::RightTop,
            Self::TopCenter | Self::BottomCenter => *self,
        }
    }

    fn flip_y(&mut self) {
        *self = match self {
            Self::TopLeft => Self::BottomLeft,
            Self::TopCenter => Self::BottomCenter,
            Self::TopRight => Self::BottomRight,
            Self::RightTop => Self::RightBottom,
            Self::RightBottom => Self::RightTop,
            Self::BottomRight => Self::TopRight,
            Self::BottomCenter => Self::TopCenter,
            Self::BottomLeft => Self::TopLeft,
            Self::LeftBottom => Self::LeftTop,
            Self::LeftTop => Self::LeftBottom,
            Self::LeftCenter | Self::RightCenter => *self,
        }
    }

    // TODO: Make sure the menu stays in bounds
    fn calc(&self, bounds: &Rectangle, overlay_bounds: &Rectangle) -> Vector {
        match self {
            Placement::TopLeft => {
                Vector::new(-overlay_bounds.width + bounds.width, -overlay_bounds.height)
            }
            Placement::TopCenter => Vector::new(
                (-overlay_bounds.width + bounds.width) / 2.0,
                -overlay_bounds.height,
            ),
            Placement::TopRight => Vector::new(0.0, -overlay_bounds.height),
            Placement::RightCenter => {
                Vector::new(bounds.width, (-overlay_bounds.height + bounds.height) / 2.0)
            }
            Placement::RightTop => {
                Vector::new(bounds.width, -overlay_bounds.height + bounds.height)
            }
            Placement::RightBottom => Vector::new(bounds.width, 0.0),
            Placement::BottomRight => Vector::new(0.0, bounds.height),
            Placement::BottomCenter => {
                Vector::new((-overlay_bounds.width + bounds.width) / 2.0, bounds.height)
            }
            Placement::BottomLeft => {
                Vector::new(-overlay_bounds.width + bounds.width, bounds.height)
            }
            Placement::LeftBottom => Vector::new(-overlay_bounds.width, 0.0),
            Placement::LeftCenter => Vector::new(
                -overlay_bounds.width,
                (-overlay_bounds.height + bounds.height) / 2.0,
            ),
            Placement::LeftTop => Vector::new(
                -overlay_bounds.width,
                -overlay_bounds.height + bounds.height,
            ),
        }
    }
}

pub struct DropDownMenu<'a, Message, Theme, Renderer> {
    // TODO: Also expose whether the content is hovered
    content: Box<dyn Fn(bool) -> Element<'a, Message, Theme, Renderer> + 'a>,
    menu: Option<Element<'a, Message, Theme, Renderer>>,
    overlay_bounds: Option<Rectangle>,
    content_cached: Option<Element<'a, Message, Theme, Renderer>>,
    placement: Placement,
    open_cached: Rc<Cell<bool>>,
    just_closed: Rc<Cell<bool>>,
    transparent: bool,
}

impl<'a, Message, Theme, Renderer> DropDownMenu<'a, Message, Theme, Renderer> {
    pub fn new(
        content: impl Fn(bool) -> Element<'a, Message, Theme, Renderer> + 'a,
        menu: Option<impl Into<Element<'a, Message, Theme, Renderer>>>,
        placement: Placement,
    ) -> Self {
        Self {
            content: Box::new(content),
            menu: menu.map(|e| e.into()),
            overlay_bounds: None,
            content_cached: None,
            placement,
            open_cached: Rc::new(Cell::new(false)),
            just_closed: Rc::new(Cell::new(false)),
            transparent: false,
        }
    }

    pub fn transparent(mut self, enabled: bool) -> Self {
        self.transparent = enabled;
        self
    }
}

impl<Message, Theme, Renderer: iced::advanced::Renderer> Widget<Message, Theme, Renderer>
    for DropDownMenu<'_, Message, Theme, Renderer>
{
    fn size(&self) -> Size<Length> {
        (self.content)(self.open_cached.get()).as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        if let Some(menu) = &mut self.menu {
            let overlay_bounds = menu
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &Limits::NONE)
                .bounds();
            self.overlay_bounds = Some(overlay_bounds);
        }
        self.content_cached = Some((self.content)(
            tree.state.downcast_ref::<State>().position.is_some(),
        ));
        self.content_cached
            .as_mut()
            .unwrap()
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        (self.content)(tree.state.downcast_ref::<State>().position.is_some())
            .as_widget()
            .draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State { position: None })
    }

    fn children(&self) -> Vec<Tree> {
        if let Some(menu) = &self.menu {
            vec![
                Tree::new((self.content)(self.open_cached.get())),
                Tree::new(menu),
            ]
        } else {
            vec![Tree::new((self.content)(self.open_cached.get()))]
        }
    }

    fn diff(&self, tree: &mut Tree) {
        if let Some(menu) = &self.menu {
            tree.diff_children(&[
                &(self.content)(tree.state.downcast_ref::<State>().position.is_some()),
                menu,
            ]);
        } else {
            tree.diff_children(&[&(self.content)(
                tree.state.downcast_ref::<State>().position.is_some(),
            )]);
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        (self.content)(tree.state.downcast_ref::<State>().position.is_some())
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        (self.content)(tree.state.downcast_ref::<State>().position.is_some())
            .as_widget_mut()
            .update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );

        if shell.is_event_captured() {
            self.just_closed.set(false);
            return;
        }

        if let Some(pos) = cursor.position()
            && layout.bounds().contains(pos)
        {
            if let Event::Mouse(mouse::Event::ButtonPressed(..)) = event {
                let state = tree.state.downcast_mut::<State>();
                if self.just_closed.get() {
                    state.position = None;
                    self.open_cached.set(false);
                } else {
                    let bounds = layout.bounds();
                    let overlay_bounds = self.overlay_bounds.unwrap_or_default();
                    let offset = self.placement.calc(&bounds, &overlay_bounds);
                    let target = bounds.position()
                        + offset
                        + Vector::new(overlay_bounds.width, overlay_bounds.height);
                    let mut placement = self.placement;
                    if bounds.width + bounds.x + offset.x < viewport.x
                        || target.x >= viewport.width + viewport.x
                    {
                        placement.flip_x();
                    }
                    if bounds.height + bounds.y + offset.y < viewport.y
                        || target.y >= viewport.height + viewport.y
                    {
                        placement.flip_y();
                    }
                    let offset = placement.calc(&bounds, &overlay_bounds);
                    state.position = Some(bounds.position() + offset);
                    self.open_cached.set(true);
                    shell.invalidate_widgets();
                }
                shell.request_redraw();
                self.just_closed.set(false);
                shell.capture_event();
            } else if !self.transparent
                && let Event::Mouse(_) = event
            {
                shell.capture_event();
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> Interaction {
        if let Some(pos) = cursor.position()
            && layout.bounds().contains(pos)
        {
            return Interaction::Pointer;
        }

        (self.content)(self.open_cached.get())
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();

        let children = if let Some(menu) = &mut self.menu {
            let [first, second] = &mut *tree.children else {
                unreachable!();
            };
            [
                // NOTE: I think this might cause issues if `content_cached` is not assigned, it won't
                // display its overlay??
                self.content_cached.as_mut().and_then(|content_cached| {
                    content_cached.as_widget_mut().overlay(
                        first,
                        layout,
                        renderer,
                        viewport,
                        translation,
                    )
                }),
                state.position.map(|position| {
                    overlay::Element::new(Box::new(Overlay {
                        menu,
                        tree: second,
                        state,
                        position: position + translation,
                        open_cached: self.open_cached.clone(),
                        just_closed: self.just_closed.clone(),
                        transparent: &self.transparent,
                    }))
                }),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
        } else {
            vec![]
        };

        (!children.is_empty()).then(|| overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message: 'a, Theme: 'a, Renderer: iced::advanced::Renderer + 'a>
    From<DropDownMenu<'a, Message, Theme, Renderer>> for Element<'a, Message, Theme, Renderer>
{
    fn from(value: DropDownMenu<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

struct Overlay<'a, 'b, Message, Theme, Renderer> {
    menu: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut Tree,
    state: &'b mut State,
    position: Point,
    open_cached: Rc<Cell<bool>>,
    just_closed: Rc<Cell<bool>>,
    transparent: &'b bool,
}

impl<Message, Theme, Renderer: iced::advanced::Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let mut layout = self
            .menu
            .as_widget_mut()
            .layout(self.tree, renderer, &Limits::new(Size::ZERO, bounds))
            .move_to(self.position);

        if bounds.width < layout.bounds().x + layout.bounds().width {
            layout.translate_mut(Vector::new(-layout.bounds().width, 0.0));
        }

        if bounds.height < layout.bounds().y + layout.bounds().height {
            layout.translate_mut(Vector::new(0.0, -layout.bounds().height));
        }

        layout
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
    ) {
        renderer.with_layer(Rectangle::INFINITE, |renderer| {
            self.menu.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout,
                cursor,
                &layout.bounds(),
            );
        });
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.menu
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let was_event_captured = shell.is_event_captured();

        self.menu.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );

        if was_event_captured {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed { .. }) => {
                if cursor.is_over(layout.bounds()) {
                    if !*self.transparent {
                        shell.capture_event();
                    }
                } else {
                    self.state.position = None;
                    self.just_closed.set(true);
                    self.open_cached.set(false);
                    shell.invalidate_widgets();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased { .. })
                if shell.is_event_captured() && cursor.is_over(layout.bounds()) =>
            {
                self.state.position = None;
                self.just_closed.set(true);
                self.open_cached.set(false);
                shell.invalidate_widgets();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::WheelScrolled { .. }) => {
                if !*self.transparent {
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                self.state.position = None;
                self.just_closed.set(true);
                self.open_cached.set(false);
                shell.invalidate_widgets();
                shell.capture_event();
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
    ) -> Interaction {
        let interaction = self.menu.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        );

        if interaction == Interaction::None && cursor.is_over(layout.bounds()) && !self.transparent
        {
            Interaction::Idle
        } else {
            interaction
        }
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.menu.as_widget_mut().overlay(
            self.tree,
            layout,
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }
}
