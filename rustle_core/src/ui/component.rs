use crate::ui::values::Color;

pub(crate) trait Component<S> {
    type Props;

    fn select(&self, state: S) -> Self::Props;
    fn render(&self, props: Self::Props) -> Element;
}

pub(crate) enum Element {
    Span(TextSpan),
    // The `Container` variant is wrapped in a `Box` to avoid a large enum variant.
    // The `Container` struct is significantly larger than other variants, so boxing it
    // reduces the overall size of the `Element` enum by storing the `Container` on the
    // heap. This improves memory efficiency as the enum only needs to store a pointer
    // instead of the full `Container` data.
    Container(Box<Container>),
}

pub(crate) struct Container {
    pub layout: taffy::Style,
    pub children: Vec<Element>,
}

pub(crate) struct TextSpan {
    pub(crate) background: Color,
    pub(crate) color: Color,
    pub(crate) text: String,
}
