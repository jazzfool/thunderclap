pub trait CoreWidget<S> {
    fn derive_state(&self) -> S;

    fn on_transform(&mut self) {}
}
