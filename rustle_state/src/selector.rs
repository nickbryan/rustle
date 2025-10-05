/// A `Selector` is a pure function used to query and derive data from the application state.
/// It takes the state as input and returns a specific piece of data, without modifying the state.
///
/// Selectors provide several benefits:
///
/// * **Encapsulation**: The logic for deriving data from the state is centralized and reusable.
/// * **Performance**: Can be memoized to avoid re-computing derived data on every state change.
/// * **Maintainability**: Simplifies components by decoupling them from the internal structure
///   of the state.
pub trait Selector<State> {
    /// The type of the data that the selector will return.
    type Result;

    /// Selects a value from the state.
    fn select(&self, state: &State) -> Self::Result;
}

/// Allow a function to be used as a selector if the function's signature matches.
impl<F, State, Result> Selector<State> for F
where
    F: Fn(&State) -> Result,
{
    type Result = Result;

    fn select(&self, state: &State) -> Self::Result {
        self(state)
    }
}
