use iced::Element;
use iced_widget::{sensor, space};

pub struct VirtualList<'a, Model, Message> {
    pub model: Vec<Model>,
    pub delegate: Box<dyn Fn(Model) -> Element<'a, Message> + 'a>,
    pub delegate_height: f32,
    pub visibilities: &'a [bool],
    pub list: Box<dyn Fn(Vec<Element<'a, Message>>) -> Element<'a, Message> + 'a>,
    pub on_show: Box<dyn Fn(usize) -> Message + 'a>,
    pub on_hide: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Model, Message> From<VirtualList<'a, Model, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: VirtualList<'a, Model, Message>) -> Self {
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
