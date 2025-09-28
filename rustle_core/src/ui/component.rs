pub trait Component {
    type Props;


    fn select<S>(&self, state: S) -> Self::Props;
    fn render(&self, props: Self::Props) -> String;
}