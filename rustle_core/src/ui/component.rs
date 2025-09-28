use crate::ui::values::Color;

pub trait _Component<S> {
    type Props;


    fn select(&self, state: S) -> Self::Props;
    fn render(&self, props: Self::Props) -> _Element;
}

pub enum _Element {
    Span(_TextSpan),
    Container(_Container)
}

pub struct _Container{
    pub layout: taffy::Style,
    pub children: Vec<_Element>
}

pub struct _TextSpan {
    pub background: Color,
    pub color: Color,
    pub text: String,
}