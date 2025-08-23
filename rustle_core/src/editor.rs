use crate::state::Store;

#[derive(Default, Clone, Debug, PartialEq)]
struct State {
    value: String,
}

enum Action {
    SetValue(String),
}

struct Editor {
    state: Store<fn(State, Action) -> State, State, Action>,
}

impl Editor {
    fn new() -> Self {
        Self {
            state: Store::new(root_reducer),
        }
    }

    pub async fn set_value(&self, value: String) {
        self.state.dispatch(Action::SetValue(value)).await;
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&State) + Send + Sync + 'static,
    {
        self.state.subscribe(callback);
    }
}

fn root_reducer(mut state: State, action: Action) -> State {
    match action {
        Action::SetValue(value) => {
            state.value = value;
        }
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_editor_subscription() {
        let editor = Editor::new();
        let received_states = Arc::new(Mutex::new(Vec::new()));

        let states_clone = received_states.clone();
        editor.subscribe(move |state| {
            println!("State received: {:?}", state);
            states_clone.lock().unwrap().push(state.clone());
        });

        // Wait for the initial state
        sleep(Duration::from_millis(10)).await;

        editor.set_value("hello".to_string()).await;
        sleep(Duration::from_millis(10)).await;

        editor.set_value("world".to_string()).await;
        sleep(Duration::from_millis(10)).await;

        let states = received_states.lock().unwrap();
        assert_eq!(states.len(), 3);
        assert_eq!(states[0].value, "");
        assert_eq!(states[1].value, "hello");
        assert_eq!(states[2].value, "world");
    }
}
