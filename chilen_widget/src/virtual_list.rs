use iced::Element;
use iced_widget::{sensor, space};

pub type ListWidget<'a, Message, Theme, Renderer> = Box<
    dyn Fn(Vec<Element<'a, Message, Theme, Renderer>>) -> Element<'a, Message, Theme, Renderer>
        + 'a,
>;

pub struct VirtualList<'a, Model, I, Message, Theme, Renderer>
where
    I: IntoIterator<Item = Model>,
{
    pub model: I,
    pub delegate: Box<dyn Fn(Model) -> Element<'a, Message, Theme, Renderer> + 'a>,
    pub delegate_height: f32,
    pub visibilities: &'a [bool],
    pub list: ListWidget<'a, Message, Theme, Renderer>,
    pub on_show: Box<dyn Fn(usize) -> Message + 'a>,
    pub on_hide: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Model, I, Message, Theme, Renderer>
    From<VirtualList<'a, Model, I, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    I: IntoIterator<Item = Model>,
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + iced::advanced::Renderer,
{
    fn from(value: VirtualList<'a, Model, I, Message, Theme, Renderer>) -> Self {
        let delegates: Vec<_> = value
            .model
            .into_iter()
            .enumerate()
            .map(|(index, model)| {
                let on_show = (value.on_show)(index);
                let on_hide = (value.on_hide)(index);
                let content = match value.visibilities.get(index).copied().unwrap_or_default() {
                    true => (value.delegate)(model),
                    false => space().height(value.delegate_height).into(),
                };
                sensor(content)
                    .on_show(move |_| on_show.clone())
                    .on_hide(on_hide)
                    .into()
            })
            .collect();
        (value.list)(delegates)
    }
}
