/// A `Reducer` is a pure function responsible for all state mutations in the application.
/// It takes the current state and an action as input, and returns a new state.
///
/// Reducers must be pure functions, meaning they have no side effects and their output
/// is solely determined by their input. This ensures that state changes are predictable
/// and traceable.
///
/// Key properties of a reducer:
///
/// * **Purity**: No side effects, such as I/O or network requests.
/// * **Immutability**: Does not modify the original state; instead, it returns a new,
///   updated state.
/// * **Composition**: Can be combined with other reducers to manage different parts of
///   the state tree.
pub trait Reducer <S, A> {
    /// The reduce function is called when an action is dispatched to the store.
    fn reduce(&self, state: S, action: A) -> S;
}

// Allow a function to be used as a reducer if the function signature matches.
impl <F, S, A> Reducer<S, A> for F where F: Fn(S, A) -> S {
    fn reduce(&self, state: S, action: A) -> S {
        self(state, action)
    }
}