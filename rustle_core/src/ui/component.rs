trait Component {
    type Props;

    fn render(&self, props: Self::Props) -> String;
}