use iced::{
    Element, Length, Size, Vector,
    advanced::{
        Widget,
        layout::{Limits, Node},
        mouse,
        widget::{Tree, tree},
    },
};

pub struct Marquee<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Marquee<'a, Message, Theme, Renderer> {
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

struct State {
    diff: f32,
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
        let state = State { diff: 0.0 };
        tree::State::new(state)
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

        let bounds = node.bounds();
        let state = tree.state.downcast_mut::<State>();
        state.diff = (bounds.width - max_bounds.width).max(0.0);

        let size = Size::new(
            bounds.width.min(max_bounds.width).max(min_bounds.width),
            bounds.height.min(max_bounds.height).max(min_bounds.height),
        );
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
        dbg!(state.diff);

        let child_viewport = iced::Rectangle {
            x: viewport.x + state.diff,
            y: viewport.y,
            width: viewport.width,
            height: viewport.height,
        };

        renderer.with_layer(layout.bounds(), |renderer| {
            renderer.with_translation(Vector::new(-state.diff, 0.0), |renderer| {
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
