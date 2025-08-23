/// A reducer is responsible for updating the state based on an action.
pub trait Reducer <S, A> {
    /// Reduce gets called when an action is dispatched to the store.
    fn reduce(&self, state: S, action: A) -> S;
}



// Allow a function to be used as a reducer if the function signature matches.
impl <F, S, A> Reducer<S, A> for F where F: Fn(S, A) -> S {
    fn reduce(&self, state: S, action: A) -> S {
        self(state, action)
    }
}