use crate::ui::values::Color;

pub trait Component<S> {
    type Props;


    fn select(&self, state: S) -> Self::Props;
    fn render(&self, props: Self::Props) -> Element;
}

pub enum Element {
    Span(TextSpan),
    Container(Container)
}

pub struct Container {
    pub layout: taffy::Style,
    pub children: Vec<Element>
}

pub struct TextSpan {
    pub background: Color,
    pub color: Color,
    pub text: String,
}