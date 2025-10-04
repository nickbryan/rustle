# Design Document: Declarative TUI Architecture

*   **Status:** Proposed
*   **Author:** Gemini
*   **Date:** 2025-09-30

### 1. Overview

This document specifies the architecture for a declarative, performant, and scalable UI framework for a terminal-based application. The design is heavily inspired by React's component model and principles, adapted to the Rust type system.

The core goal is to enable developers to build complex user interfaces by composing small, independent, and reusable components. The architecture separates concerns between data management and rendering, and it includes a memoization system to ensure high performance by avoiding unnecessary re-renders.

### 2. Architectural Principles

*   **Unidirectional Data Flow:** State flows down from a single source of truth (`AppState`). UI components react to this state and do not have their own persistent local state.
*   **Separation of Concerns:** The architecture makes a clear distinction between "Container" components that manage data and logic, and "Presentational" components that are only concerned with rendering UI.
*   **Composition over Inheritance:** The UI is built by composing components, either by nesting them or by passing components as props.
*   **Declarative API:** Developers declare *what* the UI should look like for a given state, and the framework determines *how* to update the UI efficiently.

### 3. Core Design

#### 3.1. State Management

The application has a single, global state struct (`AppState`) which acts as the single source of truth. This state is passed by immutable reference to the root of the component tree during the render cycle.

```rust
// Example AppState
pub struct AppState {
    pub user_name: String,
    pub theme_color: Color,
    // ... other global state
}
```

#### 3.2. The Component Model

The architecture is built on two distinct types of components.

##### 3.2.1. Presentational ("Dumb") Components

These are simple, reusable building blocks.

*   **Responsibilities:** To render a piece of UI based on the props it receives.
*   **Characteristics:**
    *   Does not access global state directly.
    *   Receives all data via a `props` parameter.
    *   Highly reusable and easy to test in isolation.
*   **Trait:** `PresentationalComponent<P, S>`

##### 3.2.2. Container ("Smart") Components

These are the orchestrators of the application.

*   **Responsibilities:** To connect to the global state, select the data needed for a specific UI section, and pass that data down to presentational children.
*   **Characteristics:**
    *   Accesses the global `AppState` via its `select` method.
    *   Often doesn't have complex rendering logic itself; its main job is to compose other components.
*   **Trait:** `ContainerComponent<S>`

#### 3.3. The Render Tree (`Element`)

The `render` method of a component does not draw directly to the screen. Instead, it returns a lightweight, declarative description of the UI called an `Element`. This forms a virtual UI tree, analogous to a virtual DOM.

The `Element` enum has three primary variants:
*   `Span`: A primitive for rendering styled text.
*   `Container`: A layout primitive that holds a vector of child `Element`s.
*   `Node`: A special variant that holds a nested component, enabling the component tree structure.

#### 3.4. The Rendering Engine

The rendering engine is responsible for walking the component tree, determining what has changed, and updating the UI.

##### 3.4.1. The `Renderable` Trait

To store different component types within the same tree structure (`Vec<Element<S>>`), we use a trait object: `Box<dyn Renderable<S>>`. This is the core of the type erasure mechanism. Two structs implement this trait:
*   `PresentationalNode`: Wraps a `PresentationalComponent` and the `props` it was given by its parent.
*   `ContainerNode`: Wraps a `ContainerComponent`.

##### 3.4.2. Recursive Traversal

The rendering process is a top-down, recursive traversal of the component tree, starting from the root node. For each `Element::Node`, the engine calls the `render_and_cache` method on its `Renderable` trait object.

##### 3.4.3. Component Identification (`ComponentId`)

To enable memoization, every component instance in the tree needs a stable, unique ID. This is generated based on the component's path from the root (e.g., `[0]` for the root, `[0, 1]` for the second child of the root).

#### 3.5. Memoization

To ensure high performance, the framework avoids re-rendering components whose props have not changed.

*   **`MemoizationCache`:** A struct holding two HashMaps keyed by `ComponentId`:
    1.  `props_cache`: Stores a `Box<dyn Any>` containing the props from the last render.
    2.  `element_cache`: Stores the `Element` tree returned by the last render.
*   **`should_render` Logic:** Before rendering a component, the engine:
    1.  Retrieves the old props from the `props_cache`.
    2.  Compares the old props with the new props using `PartialEq`.
    3.  If they are identical, the render is skipped, and the previously rendered `Element` is retrieved from the `element_cache`. This halts the recursive render for the entire branch.

#### 3.6. Composition: The "Slot" Pattern

To maintain a clean separation of concerns, a presentational component should not directly render a container component. Instead, the recommended pattern is for the presentational component to accept a generic `Element` as a prop. This creates a "slot" that the parent container can fill with any content, including another container component. This pattern is analogous to `props.children` in React and is fundamental to building a flexible and composable UI.

### 4. Code Definitions

This section contains the complete definitions for the core types and traits.

<details>
<summary>Click to view Core Code Definitions</summary>

**`rustle_core/src/ui/component.rs`**
```rust
use crate::ui::element::Element;

pub trait PresentationalComponent<P, S>
where
    P: PartialEq + Clone + 'static,
{
    fn render(&self, props: P) -> Element<S>;
}

pub trait ContainerComponent<S> {
    type Props: PartialEq + Clone + 'static;
    fn select(&self, state: &S) -> Self::Props;
    fn render(&self, props: Self::Props) -> Element<S>;
}
```

**`rustle_core/src/ui/element.rs`**
```rust
use crate::ui::render::Renderable;
use taffy::Style;

pub enum Element<S> {
    Span(TextSpan),
    Container(ContainerElement<S>),
    Node(Box<dyn Renderable<S>>),
}

pub struct ContainerElement<S> {
    pub layout: Style,
    pub children: Vec<Element<S>>,
}

pub struct TextSpan { /* ... */ }
```

**`rustle_core/src/ui/render.rs`**
```rust
use crate::ui::component::{ContainerComponent, PresentationalComponent};
use crate::ui::element::Element;
use crate::ui::memoize::{should_render, ComponentId, MemoizationCache};
use std::marker::PhantomData;

pub trait Renderable<S> {
    fn render_and_cache(&self, state: &S, cache: &mut MemoizationCache<S>, id: ComponentId) -> Element<S>;
}

pub struct PresentationalNode<C, P, S> where C: PresentationalComponent<P, S>, P: PartialEq + Clone + 'static { /* ... */ }
impl<C, P, S> Renderable<S> for PresentationalNode<C, P, S> where C: PresentationalComponent<P, S> + 'static, P: PartialEq + Clone + 'static, S: 'static { /* ... */ }

pub struct ContainerNode<C, S> where C: ContainerComponent<S> { /* ... */ }
impl<C, S> Renderable<S> for ContainerNode<C, S> where C: ContainerComponent<S> + 'static, S: 'static { /* ... */ }

pub fn presentational_node<C, P, S>(component: C, props: P) -> Element<S> where C: PresentationalComponent<P, S> + 'static, P: PartialEq + Clone + 'static, S: 'static { /* ... */ }
pub fn container_node<C, S>(component: C) -> Element<S> where C: ContainerComponent<S> + 'static, S: 'static { /* ... */ }
```

**`rustle_core/src/ui/memoize.rs`**
```rust
use crate::ui::element::Element;
use std::any::Any;
use std::collections::HashMap;

pub type ComponentId = Vec<u16>;

pub struct MemoizationCache<S> {
    pub props_cache: HashMap<ComponentId, Box<dyn Any>>,
    pub element_cache: HashMap<ComponentId, Element<S>>,
}

pub fn should_render<P, S>(cache: &mut MemoizationCache<S>, id: &ComponentId, new_props: &P) -> bool
where
    P: PartialEq + Clone + 'static,
{
    // ... logic as defined previously
}
```
</details>

### 5. Example Usage

This example demonstrates a smart `AppContainer` selecting state and passing it as props to a dumb `Greeting` component.

<details>
<summary>Click to view Example Usage</summary>

```rust
// 1. The global state
pub struct AppState {
    pub user_name: String,
}

// 2. The dumb component's props
#[derive(PartialEq, Clone)]
pub struct GreetingProps {
    pub name: String,
}

// 3. The dumb component
pub struct Greeting;
impl PresentationalComponent<GreetingProps, AppState> for Greeting {
    fn render(&self, props: GreetingProps) -> Element<AppState> {
        Element::Span(TextSpan { text: format!("Hello, {}!", props.name), ..Default::default() })
    }
}

// 4. The smart component
pub struct AppContainer;
impl ContainerComponent<AppState> for AppContainer {
    type Props = String; // Selects the user name string

    fn select(&self, state: &AppState) -> Self::Props {
        state.user_name.clone()
    }

    fn render(&self, props: Self::Props) -> Element<AppState> {
        // `props` is the string selected from the state.
        // It creates props for its child...
        let greeting_props = GreetingProps { name: props };
        // ...and renders the child.
        Element::Container(ContainerElement {
            children: vec![
                presentational_node(Greeting, greeting_props)
            ],
            ..Default::default()
        })
    }
}
```
</details>

### 6. Future Considerations

*   **User Input:** A system for propagating user input events (e.g., key presses) up the tree or as actions to a central dispatcher will be required.
*   **List Rendering:** For dynamically sized lists of children, a `key` property (similar to React's) should be introduced to ensure `ComponentId`s remain stable across renders, preventing state loss and incorrect memoization.
*   **Asynchronous Actions:** A mechanism for components to dispatch asynchronous actions (e.g., loading a file) and update the `AppState` upon completion will be necessary for real-world applications.
