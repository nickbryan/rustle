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
pub trait Reducer<S, A> {
    /// The reduce function is called when an action is dispatched to the store.
    ///
    /// # Design Rationale for Passing Action by Value
    ///
    /// The `action` is passed by value (`A`), which simplifies the ownership model.
    /// This approach is efficient when actions are small and cheap to copy, such as simple
    /// enums or structs with primitive types. By taking ownership, the reducer can consume
    /// or store the action without lifetime complexities.
    ///
    /// However, if `A` were a large data structure, passing it by value could lead to
    /// performance overhead due to copying. In such scenarios, an alternative design
    /// might pass the action by reference (`&A`). This would avoid copies but introduce
    /// lifetime annotations and management, making the trait and its implementations
    /// more complex.
    ///
    /// The current design prioritizes simplicity. If the application evolves to a point
    /// where action-cloning becomes a performance bottleneck, this trait definition
    /// should be revisited.
    fn reduce(&self, state: S, action: A) -> S;
}

// A type alias for a function that implements the `Reducer` trait.
pub type ReducerFn<S, A> = fn(S, A) -> S;

// Allow a function to be used as a reducer if the function signature matches.
impl<F, S, A> Reducer<S, A> for F
where
    F: Fn(S, A) -> S,
{
    fn reduce(&self, state: S, action: A) -> S {
        self(state, action)
    }
}
