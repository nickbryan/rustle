use crate::ui::values::Color;

/// A trait for components that can be rendered in the UI.
/// Components are responsible for selecting their props from the state and rendering an element.
pub(crate) trait Component<S> {
    /// The properties required by the component to render.
    type Props;

    /// Selects the component's props from the given state.
    fn select(&self, state: S) -> Self::Props;
    /// Renders the component with the given props.
    fn render(&self, props: Self::Props) -> Element;
}

/// Represents a UI element that can be rendered.
pub(crate) enum Element {
    Span(TextSpan),
    // The `Container` variant is wrapped in a `Box` to avoid a large enum variant.
    // The `Container` struct is significantly larger than other variants, so boxing it
    // reduces the overall size of the `Element` enum by storing the `Container` on the
    // heap. This improves memory efficiency as the enum only needs to store a pointer
    // instead of the full `Container` data.
    Container(Box<Container>),
}

/// A container element that can hold other elements.
/// The layout of the container and its children is determined by the `layout` and `children` fields.
pub(crate) struct Container {
    pub layout: taffy::Style,
    pub children: Vec<Element>,
}

/// A text span element with a background color, foreground color, and text content.
pub(crate) struct TextSpan {
    pub(crate) background: Color,
    pub(crate) color: Color,
    pub(crate) text: String,
}
