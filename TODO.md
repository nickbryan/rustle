# TODO

Bugs should be fixed first. Architecture should be tackled next — most of the
Feature tasks are downstream of those decisions. Tooling (tests, tracing) is
worth landing in parallel because it's needed to validate everything else.

---

## Bugs

### Blank screen on startup until first keypress

**Problem.** `Editor::consume` only redraws inside the `state_rx.changed()`
branch of the `select!` (rustle_core/src/editor.rs:99). The watch channel's
initial value, set in `watch::channel(state.clone())` in `Actor::new`
(rustle_state/src/actor.rs:45), does not count as a `send` — so a fresh
subscriber does not see `changed()` fire until the first actual mutation.
The editor opens with a blank terminal until the user presses any key that
triggers a dispatch.

**Approach.** Either render once explicitly before entering the loop, or
have the actor call `notifier.send(state.clone())` immediately after
starting its `act()` loop. Rendering once at the top of `Editor::consume`
is the smaller change; sending an initial state from the actor is the more
honest one because it also helps any future subscriber that's not the
editor's own render loop.

**Acceptance.**
- Running the binary shows the status line and command line immediately,
  without requiring a keypress.
- Add an integration test (see Tooling) that subscribes to a fresh store
  and asserts the initial state is observable without a prior dispatch.

---

### Resolver consumes digits that could belong to a chord

**Problem.** `parse_multiplier` (rustle_core/src/input.rs:199) eats any
digit when `multiplier > 0`. Any chord containing digits becomes
unreachable with a count prefix: `5g0` (count 5, chord `g0`) cannot work
because `0` is always consumed as the second digit of the multiplier.
Niche today — no digit-containing chords ship — but real the moment one is
added.

**Approach.** Treat digits as multiplier input only when the resolver
buffer is empty. Once any key has been added to the buffer (we're inside
a chord), all subsequent keys go through binding lookup. This is the
simplest rule and matches Vim's behaviour for prefixed chords.

**Acceptance.**
- A chord like `g0` works on its own and with a count.
- Existing multiplier behaviour (`5j`, `30j`) is unchanged.
- Add a Resolver unit test for both cases (see Tooling).

---

### `Config::default()` panics; `Error::Config` is unreachable

**Problem.** `Config::default()` (rustle_core/src/config.rs:18) does
`toml::from_str(DEFAULT_CONFIG_TOML).expect(...)`. The default TOML is
baked in via `include_str!`, so this never fails in practice — but the
`Error::Config` variant exists for this case and isn't wired up anywhere.

**Approach.** Wire `Error::Config` through a real fallible config-loading
path. This converges with the "User config loading" task in Tooling, which
is where parse errors naturally become real and recoverable. Until that
lands, document the `expect` with a clear "infallible because compile-time
input" comment or replace it with a `panic!` that says exactly that.

**Acceptance.**
- Either `Error::Config` is plumbed through a real loading path, or the
  variant is removed and the panic is documented as intentional.

---

## Architecture

### Add a side-effect / middleware layer for async I/O

**Problem.** Reducers are pure (`rustle_state::Reducer::reduce` is sync, `&self`),
and the actor only knows about `Dispatch` and `Select`. There is currently
nowhere for async or side-effectful work (file I/O, LSP, clipboard, async
network) to live. The moment `:e <path>`, `:w`, undo/redo persistence, or
anything else with side effects lands, this gap will dominate the design.

**What to build.** Choose one of:

1. *Middleware* — a chain that sees actions before the reducer runs and can
   dispatch follow-up actions (Redux middleware analogue). Simple, but only
   helpful for synchronous interception unless combined with #2.
2. *Effects / thunks* — an action variant that carries an `async fn(Store) -> ()`
   the actor (or a dedicated effects actor) spawns. Best ergonomics for
   "load file, dispatch BufferLoaded(text)" flows.
3. *Effect actor* — a second actor receiving `Effect` messages, dispatching
   back into the store on completion. Cleanest separation, most plumbing.

Recommend option 2 to start: add `Action::Effect(BoxFuture<Action>)` or a
parallel `Store::dispatch_effect` path that uses the injected `Runtime`. Keep
reducers pure.

**Acceptance.**
- A buffer can be loaded from disk via an action without `Editor::consume`
  knowing about files.
- Errors from effects surface via dispatched actions (e.g. `FileLoadFailed`),
  not panics or `Result` returns.
- Works under both `TokioRuntime` and `WasmRuntime`.

---

### Move resolver state into the Store (single source of truth)

**Problem.** `Resolver { buffer, multiplier }` and the `pending_timeout` flag
in `Editor::consume` are the only mutable application state living outside
the Store. This directly blocks the "show current chord / multiplier" task —
rendering needs to see it. It also means input resolution can't be tested via
the standard dispatch-and-select pattern.

**Approach.**
- Promote pending input to state: add a `pending_input: PendingInput` field
  to `component::root::State` (or a new `component::input` module) containing
  the current chord buffer, multiplier, and pending-timeout flag.
- Convert each keypress into an action (e.g. `Action::Key(Key)`) dispatched
  through the store. The reducer runs the resolution logic and emits the
  resolved action via the effects layer above (or via a returned
  `Vec<Action>`, if you prefer immediate sequencing).
- `Editor::consume` becomes a thin event loop that just dispatches `Key`
  events and timeouts; resolution lives in a reducer.

**Acceptance.**
- The resolver no longer holds mutable state — it's a pure function from
  `(PendingInput, Key, Mode, &Bindings) -> (PendingInput, Resolution)`.
- A `pending` component can render the current buffer + multiplier with the
  same `(&State) -> Element` shape as every other component.
- `Editor` owns no mutable resolution state.

---

### Notifier should fire only on real state changes

**Problem.** `Actor::deliver` for `Dispatch` (rustle_state/src/actor.rs:84) calls
`self.notifier.send(new_state)` unconditionally. Combined with `Viewport::redraw`
clearing and rebuilding the entire `TaffyTree` (rustle_core/src/ui/render.rs:297),
every no-op dispatch costs a full layout pass.

**Approach.**
- Derive `PartialEq` on `State` and all component sub-states.
- Use `watch::Sender::send_if_modified` (or compare before send) so subscribers
  only wake on actual changes.

**Acceptance.**
- Dispatching an action that doesn't change state does not trigger a redraw.
- Existing tests still pass; add a unit test asserting no notification on a
  no-op dispatch.

---

### Complete the `Runtime` abstraction (time, not just spawn)

**Problem.** `Editor::consume` uses `tokio::time::timeout` directly
(rustle_core/src/editor.rs:64) even though the whole point of the `Runtime`
trait is to keep `rustle_core` runtime-agnostic. Already flagged with a
`// TODO` in the code. WASM idle-timeout currently can't work.

**Approach.**
- Extend `Runtime` with `async fn sleep(&self, Duration)` and a
  `timeout<F: Future>(&self, Duration, F)` helper (or expose a `Timer` trait
  separately if cleaner).
- Implement for `TokioRuntime` (tokio::time::sleep) and `WasmRuntime`
  (gloo-timers or wasm-bindgen `setTimeout`).
- Replace direct `tokio::time` usage in `editor.rs`.

**Acceptance.**
- `rustle_core` has no `tokio::time` references.
- Insert-mode chord idle timeout works in the web build.

---

### Consolidate action routing and reducer signatures

**Problem.** `component::root::reduce` (rustle_core/src/component/root.rs:28)
mixes three concerns: (1) handling root-level actions, (2) gating
sub-component reducers by mode, (3) handling `Cancel` again after the
sub-reducer already saw it. `command_line::reduce` takes `(C, &A) -> C` while
the root takes `(S, A) -> S`. `command_line::reduce` also clobbers `text` to
`":"` on every action while in command mode — works incidentally, not by
intent.

**Approach.**
- Pick one signature convention. Recommend `(S, &A) -> S` everywhere (action
  passed by reference) and revisit the `Reducer` trait's by-value docstring
  rationale — by-value made sense when actions were `Copy`-like, but the
  resolver work above will likely make `Action` carry owned data
  (`InsertString(String)`).
- Decide on a routing convention. Two options worth weighing:
  - *Pass-through*: every component reducer sees every action; they decide
    what to handle. Simple, scales until many components.
  - *Root-dispatched*: root explicitly routes actions to sub-reducers.
    Explicit, easier to reason about, more boilerplate.
- Remove the duplicate `Cancel` handling — handle it in exactly one place.
- Drop the "set text = `:` on every action in command mode" pattern; only
  set it on `EnterMode(Command)`.

**Acceptance.**
- One reducer signature across the codebase.
- Each action is handled in exactly one well-defined location.
- Document the chosen routing convention in `rustle_core/README.md`.

---

### Decide on the "actor model" framing

**Problem.** `rustle_state/README.md` markets an actor system, but there's
exactly one actor (the store). The `Mailbox`/`Address`/`Deliver` machinery
isn't exposed publicly, so no consumer can spawn another actor anyway.

**Pick one.**
- *Retire the framing*: rename internally and in the README to something
  honest like "Redux store on a dedicated task". Keep the implementation as-is.
- *Lean in*: expose `Mailbox`/`Address`/`Deliver` publicly, add docs/examples
  for building auxiliary actors (e.g. an effects actor — see the side-effect
  task above), and use a second actor somewhere real to validate the design.

The choice depends on whether the effects layer above will be implemented as
a second actor. If yes, lean in. If as middleware/thunks, retire the framing.

**Acceptance.**
- README and code framing agree.
- If "lean in": at least one consumer of the public actor API exists in the
  codebase.

---

### Consider per-mode state via a tagged union

**Problem.** `State` carries `command_line: CommandLine` permanently, even in
Normal/Insert mode. As modes grow (visual, replace, search), `State` becomes
a flat product of every mode's data. Some fields will always be dead.

**Approach.** Replace `mode: Mode` + flat fields with something like:

```rust
enum ModeState {
    Normal,
    Insert { /* … */ },
    Command(CommandLine),
}
```

Trade-off: more verbose pattern-matching in reducers and renderers, but
mode-specific state can't accidentally leak across modes, and dead fields
are impossible by construction.

**Not urgent** — flag this when the third mode-specific component lands.
Wait for the resolver-in-store work first, since that adds another
state-shape decision to make in tandem.

---

### Keep `Action` serialisable and replayable

**Why.** Pure reducers + a typed action stream is a perfect substrate for
macros, undo/redo via action-log replay, and time-travel debugging. The
moment an action carries a `BoxFuture`, an `Arc<dyn Trait>`, or a closure,
that substrate is gone — and rolling it back is painful.

**Constraint to hold.** As `Action` grows (Buffer editing, File I/O,
Effects), keep variants owned-data-only: no function pointers, no futures,
no closures. The effects layer (first Architecture task) should carry
effects via a *separate* message type or via an `EffectId` that resolves
to a registered handler — not as a `Future`-bearing `Action` variant.

**Acceptance.**
- `Action` (in `rustle_core::input`) can derive `serde::Serialize` /
  `Deserialize` once owned-only variants land. Don't add the derives until
  there's a consumer for them (macros / replay).
- The Architecture decision on effects is consistent with this constraint
  and explicitly documented.

---

## Core features (currently stubbed)

### Implement buffer text editing

**Current state.** `component::buffer::Buffer { text: Rope }` exists but
`buffer::reduce` is a no-op and the `action` parameter is unused. `render`
emits an empty `TextSpan`. The `Rope` is never written to.

**Tasks.**
- Define actions for text mutation: `InsertString(String)`, `InsertChar(char)`,
  `DeleteBackward`, `DeleteForward`, `DeleteLine`, etc. (Start with insert and
  backspace; grow as needed.)
- Implement `buffer::reduce` to apply these to the `Rope` at the cursor
  position. Cursor position currently lives on `State::cursor_position` —
  decide whether it stays on root state or moves into `Buffer` (probably the
  latter; cursor is per-buffer).
- Update `buffer::render` to emit one `TextSpan` per line (or per visible
  line, when scrolling lands). Respect the rendered area's height/width.
- Re-enable the resolver buffer-drain → `InsertString` path in
  `Editor::consume` (currently commented out at editor.rs:81 and editor.rs:91).

**Acceptance.**
- Typing in insert mode appears on screen.
- Backspace deletes the previous grapheme (not byte — use
  `unicode-segmentation`).
- Unicode and multi-width characters render correctly (already handled in
  `Frame::write`).

---

### Wire up cursor movement actions

**Current state.** Default config (rustle_core/src/defaults/config.toml)
binds `h/j/k/l` to `move_cursor_prev`, `move_line_next`, etc., but
`input::parse_action` (rustle_core/src/input.rs:214) doesn't recognise any
of them — those keypresses currently no-op.

**Tasks.**
- Add `Action::MoveCursor { direction: Direction, count: u32 }` (or four
  separate variants — direction-as-data scales better with `<count>` repeats).
- Map all four config strings in `parse_action`, threading the multiplier
  through as the count.
- Implement the reducer logic: bounded by line length / buffer line count,
  graphemes not bytes.
- Movement is per-buffer/window, so it probably lives in
  `component::buffer::reduce` once buffer state is real.

**Acceptance.**
- `h/j/k/l` move the cursor in normal mode.
- `5j` moves down 5 lines (multiplier already parses correctly per recent
  commits).
- Cursor cannot leave the buffer bounds.

---

### Implement command-mode input and execution

**Current state.** `command_line::reduce` sets `text = ":"` on any action
while in command mode (a stub); there's no way to type into the prompt or
execute a command.

**Tasks.**
- While in command mode, route `KeyPressed(Key::Char(c))` to append to
  `command_line.text` (most cleanly via a `CommandLineInput(char)` action,
  emitted instead of the usual binding resolution when in command mode).
- `Key::Backspace` deletes the last char (stop at the leading `:`).
- `Key::Enter` parses and executes `command_line.text[1..]`. Initial command
  set: `q` (quit), `w` (write), `wq`, `e <path>` (edit/load).
- `Key::Esc` clears `text` and returns to Normal mode (this part already
  works via `Cancel`).
- File commands (`w`, `e`) need the effects layer from the Architecture
  section — do that first, or stub with `unimplemented!()` until it lands.

**Acceptance.**
- Typing `:q<Enter>` quits.
- Typing `:` then `<Esc>` returns to normal mode with the prompt cleared.
- Mistyped commands display an error in the command line (e.g. `:foo<Enter>`
  shows `unknown command: foo`).

---

### Add file loading (and saving)

**Current state.** No filesystem I/O exists anywhere. Editor only edits an
in-memory empty buffer.

**Tasks.**
- Accept an optional path argument in `rustle_tui::main` (use `std::env::args`
  or `clap` — `clap` only if argument parsing grows).
- Implement `Action::LoadFile(PathBuf)` and `Action::WriteFile(Option<PathBuf>)`
  via the effects layer.
- On successful load, dispatch `BufferLoaded { path, text }`; on failure,
  `BufferLoadFailed { path, error }` → surface in command line / status line.
- WebUI path-loading: defer; the web build doesn't have a real filesystem.
  Either disable the commands in WASM or wire them to an in-browser
  storage backend later.

**Acceptance.**
- `cargo run --bin rustle_tui -- README.md` opens the file with text visible.
- `:w` writes the buffer back to its source path.
- `:w newfile.txt` writes to a new path.

---

### Handle `WindowResized` events

**Current state.** `Event::WindowResized(u16, u16)` is defined and emitted
from the crossterm backend, but `Editor::consume` falls through `_ => ()`
for it (editor.rs:97). `Viewport::area` is captured once at construction
(render.rs:271). Resizing the terminal during a session leaves the UI
laid out for the old size.

**Tasks.**
- Plumb resize events into the Viewport: either dispatch an action that
  updates a `viewport_size` field on state and have the render path pick
  it up, or expose a `Viewport::resize(Rect)` method called directly from
  the consume loop.
- Reset both `frames[0]` and `frames[1]` to the new size (current `Frame`
  diff requires equal-area frames — see `Frame::diff` debug_assert).
- Test with both terminal resize and (for the web build) the `FitTerminalAddon`
  resize path — webui currently captures `cols/rows` once at construction in
  `rustle_webui/src/main.rs:115`.

**Acceptance.**
- Resizing the terminal during a session re-lays out the UI correctly with
  no garbage cells.
- Web build resizes correctly when the browser window changes.

---

### Implement undo / redo

**Why now.** This is interesting *because* of the redux design — there are
two natural shapes and the choice affects buffer storage. Decide the shape
before buffer editing logic settles, so retrofits aren't needed.

**Two designs.**
- *State snapshot stack*: clone the relevant sub-state (e.g. `Buffer`)
  before each mutating dispatch; undo pops. Cheap with `Rope` because clones
  are O(1) via internal Arc sharing, but you store one snapshot per edit.
- *Action log + replay*: store the sequence of mutating actions; undo
  rebuilds state by replaying from an earlier snapshot up to N-1 actions.
  Smaller; requires actions to be deterministic and self-contained (the
  "Keep `Action` serialisable" constraint above).

**Tasks.**
- Pick one. Recommend action-log for the macro overlap (see Architecture).
- Decide what counts as an "undo boundary" — per-keystroke is annoying;
  per-insert-session is the Vim convention.
- Mark non-undoable actions (cursor movement, mode change) so the log
  doesn't bloat with no-ops.

**Acceptance.**
- `u` in normal mode reverts the last edit boundary.
- `<C-r>` redoes.
- Cursor movement is not undoable.

---

### Multiple buffers, windows, and splits

**Hint in the codebase.** The recent `buffer_view.rs` → `window.rs` rename
suggests this is intended. State currently models a single implied buffer
with no concept of multiple windows.

**Tasks.**
- Promote state from "one buffer" to a `BufferStore` (`BufferId → Buffer`)
  and a `WindowTree` (a recursive split tree whose leaves point at buffer
  ids). Active window is an id into the tree.
- Splits: `:split` (horizontal), `:vsplit` (vertical).
- Navigation: `<C-w>h/j/k/l` for window focus, `<C-w>c` to close.
- Buffer management: `:bn`, `:bp`, `:bd`. Buffer picker UI later.
- Rendering is mostly free: `taffy` flexbox containers map directly onto
  the window tree once state is structured.
- Surfaces the action routing question (Architecture) again — `MoveCursor`
  needs to know which window it applies to. Either the action carries a
  target id, or routing dispatches based on active window.

**Acceptance.**
- `:split README.md` opens README.md in a new horizontal split.
- `<C-w>j` moves focus between splits.
- Closing the last window quits.

---

## Input system (existing TODOs, expanded)

### Show pending chord / multiplier

**Goal.** While the resolver is mid-chord (`Resolution::Pending`) or
accumulating a multiplier, the user should see what's pending — typically
rendered at the right edge of the status line (Vim's `showcmd`).

**Dependency.** Requires "Move resolver state into the Store" first —
otherwise the renderer can't see the pending input. Once that's done:

**Tasks.**
- Add a `pending` render slot in `status_line::render` (right-aligned) that
  formats `PendingInput` as `<multiplier?><buffered keys>`. Examples:
  `5` (just multiplier), `12d` (multiplier + first key of chord), `dd`
  (chord buffer with no multiplier).
- Use `Key::Display` impl (already exists and handles `<space>`, `<C-x>`, etc.).
- Clear automatically when the resolver resets (already implied if it reads
  live state).

**Acceptance.**
- Pressing `5` in normal mode shows `5` in the status line.
- Pressing `5d` shows `5d`.
- A resolved or cancelled chord clears the display.

---

### Allow a key to be both a chord prefix and a leaf action

**Goal.** Same key should be able to fire an action on its own *and* serve
as the first key of a chord. E.g. `j` moves down, `jj` exits insert mode.
Currently `KeyBinding::Action` and `KeyBinding::Chord` are mutually exclusive
variants in `input.rs:113`, and the resolver picks `Chord` over `Action`
when both could apply.

**Tasks.**
- Extend `KeyBinding` to allow both: e.g.
  ```rust
  enum KeyBinding {
      Action(String),
      Chord(KeyBindingMap),
      Both { action: String, chord: KeyBindingMap },
  }
  ```
  Or a struct with `action: Option<String>` and `next: Option<KeyBindingMap>`.
- Update TOML deserialization. Serde-untagged on the current shape doesn't
  cleanly express "both" — likely need a custom Deserialize impl or a
  tagged form like `{ action = "...", chord = { ... } }`.
- Update `Resolver::resolve`: on a key that matches a `Both` entry, return
  `Pending` (waiting to disambiguate via idle-timeout or next key), and on
  timeout, fall back to the leaf action with the current multiplier.
- This naturally requires the idle-timeout to apply in normal mode too, not
  just insert. Reconcile with current `editor.rs:74` logic.

**Acceptance.**
- Config can express both `j = "move_line_next"` and `j.j = "enter_normal_mode"`
  simultaneously (or whatever syntax falls out).
- Pressing `j` then waiting `idle_timeout` triggers `move_line_next`.
- Pressing `jj` quickly triggers `enter_normal_mode`.

---

### Per-binding timeouts

**Goal.** Some chords are obvious and want a long timeout (e.g. complex
prefixes like leader keys); others should be near-instant. A single global
`idle_timeout` is too coarse.

**Tasks.**
- Extend the binding schema to allow per-entry timeout overrides:
  ```toml
  [bindings.normal]
  j = { j = { action = "enter_normal_mode", timeout = 500 } }
  ```
  Or a parallel `[bindings.normal.timeouts]` table. Pick whichever survives
  the "both prefix and action" change above without exploding in complexity.
- `PendingInput` (in state, after the resolver-in-store work) should carry
  the effective timeout for the *current* pending chord, so `Editor::consume`
  knows how long to wait.
- Default falls back to `editor.idle_timeout`.

**Acceptance.**
- Per-chord timeout overrides parse from config and take effect.
- Default behaviour is unchanged when no override is set.

---

### Binding descriptions

**Goal.** Each binding can carry a human-readable description, surfaced in
a future "which-key" / cheatsheet UI.

**Tasks.**
- Extend the binding schema:
  ```toml
  [bindings.normal]
  i = { action = "enter_insert_mode", desc = "Enter insert mode" }
  ```
- Currently `KeyBinding::Action(String)` is bare. Either wrap in a struct
  or use a `serde(untagged)` form that accepts both `"action_name"` and
  `{ action = "...", desc = "..." }`. Untagged is convenient but harder to
  combine with the per-timeout and "both" changes above — consider designing
  the new binding shape *once* to cover all four extensions.
- No UI yet — just thread the description through so it's available on the
  `Resolver`/`PendingInput`.

**Acceptance.**
- Descriptions parse from config without breaking the existing config.
- Descriptions are accessible to renderers (e.g. via state) once needed.

---

### Decide on nom for command-mode parsing

**Question.** Do we need `nom`? Suspicion is no — command-mode grammar is
likely simple enough (whitespace-separated tokens, single command name,
optional args). The Vim ex-command grammar is *not* simple (ranges, regex
ranges, multi-command lines), but we don't need full ex-compat.

**Decision criteria.**
- If we target `:q`, `:w [path]`, `:e <path>`, `:set foo=bar`, `:source <path>`
  → handwritten `split_whitespace`-based parser is fine.
- If we want ranges (`:10,20s/foo/bar/g`), command chaining (`:w | q`), or
  similar → nom or `winnow` pays for itself.

**Action.** Start without nom. If grammar grows non-trivial, revisit.
Closes the existing TODO question.

---

## Tooling and project hygiene

### Add a real test suite

**Current state.** Unit tests exist in `rustle_core::ui::values` and
`rustle_core::ui::render`. Nothing in `rustle_state`, nothing for
`input::Resolver`, no reducer tests, no integration tests. The Resolver in
particular is non-trivial — chords, multipliers, idle timeouts, modes —
and untested.

**Priorities.**
1. **`input::Resolver`** — table-driven tests covering: simple action,
   nested chord, multiplier, leading-zero behaviour, idle-timeout in
   insert mode, chord-prefix-and-action (after the feature lands),
   digit-collision case (after that bug is fixed), mode switching
   mid-chord, NoMatch resets buffer cleanly.
2. **`rustle_state::Store`** — dispatch updates state, select reads state,
   subscribe receives changes, multiple subscribers see the same value,
   `StateError::ActorTerminated` returned when actor task is cancelled.
3. **Reducer tests** for each component reducer (after they have real
   logic — currently `buffer::reduce` is a no-op).
4. **Integration**: spin up an `Editor` with a `MockCanvas` and a scripted
   `EventStream`, assert the final state and the cells drawn.

**Acceptance.**
- `cargo test --workspace` runs substantially more than the current
  handful of UI tests.
- Resolver has at least 10 scenario tests.
- Store has tests for dispatch, select, subscribe, and termination.

---

### Add structured logging via `tracing`

**Why.** The actor + future effects layer + async resolver work all need
observability. `tracing` spans on dispatch (with the action variant), on
effect lifecycle, and on resolver transitions are basically free when no
subscriber is attached and invaluable when one is.

**Tasks.**
- Add `tracing` to `rustle_state` and `rustle_core`.
- Wrap `Actor::act` body in a span; instrument `Deliver` impls with the
  message variant name.
- Skip instrumentation inside per-cell rendering paths (`Frame::write`,
  `render_element`) — too hot, would bloat logs.
- TUI binary: `tracing-subscriber` writing to a file (terminal is in use).
  Filter via `RUSTLE_LOG` env var. Default file path:
  `dirs::cache_dir()/rustle/rustle.log`.
- Web binary: `tracing-wasm` writing to the browser console.

**Acceptance.**
- `RUSTLE_LOG=debug cargo run --bin rustle_tui` writes a usable trace log.
- No traces in tight rendering loops.

---

### Load config from disk

**Current state.** Default config is `include_str!`'d in `config.rs:15`.
No user config path exists; bindings and `idle_timeout` can't be changed
without recompiling.

**Tasks.**
- Read from a standard path via the `dirs` crate:
  `dirs::config_dir()/rustle/config.toml` — Linux:
  `~/.config/rustle/config.toml`, macOS:
  `~/Library/Application Support/rustle/config.toml`.
- Fall back to the bundled default if the file is missing.
- Bubble parse errors via `Error::Config` — this is what finally makes the
  variant reachable (see Bugs).
- Web build: no filesystem. Gate the disk-load path with
  `cfg(not(target_arch = "wasm32"))` and keep WASM on the bundled default.
- Consider: should the user config merge with defaults, or replace them
  wholesale? Vim-style merge is more useful but harder to reason about.
  Recommend wholesale replace initially.

**Acceptance.**
- TUI: editing `~/.config/rustle/config.toml` rebinds keys without
  rebuilding.
- Invalid TOML produces a clear error message at the command line (or
  exits with one — pick a behaviour and document it).

---

### CI, MSRV, and basic hygiene

**Gaps.**
- No CI configuration.
- No MSRV declared in `Cargo.toml`. Edition 2024 implies fairly recent
  Rust but it's unwritten.
- No `cargo deny` / `cargo audit`.
- Lint/format settings exist (`rustfmt.toml`, `clippy::pedantic` warns)
  but nothing enforces them.

**Tasks.**
- GitHub Actions workflow running:
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`
  - `cargo check --target wasm32-unknown-unknown -p rustle_webui`
- Declare `rust-version` in each crate's `Cargo.toml`.
- Optional: `cargo deny` for license / duplicate / advisory checks.

**Acceptance.**
- PRs run CI; failures block merge.
- A Rust below MSRV produces a clean error rather than a confusing one.

---

## Cleanup / quality

### Replace `async-trait` with native async fn in traits

**Why.** Workspace is on edition 2024, which supports native `async fn` in
traits. The `async-trait` macro adds per-call `Box::pin` allocations and a
build-time cost.

**Where.** `rustle_state/src/mailbox.rs` (`Deliver`, `Assign`),
`rustle_state/src/actor.rs` (impls).

**Acceptance.**
- `async-trait` removed from `rustle_state/Cargo.toml`.
- All tests pass; no behavioural change.
- If `dyn Trait` requirements force returning `Box<dyn Future>` manually
  somewhere, document why.

---

### Remove the `Actor.state: Option<S>` invariant

**Problem.** `actor.rs:87` does `self.state.take().expect("State should always
be Some")`. If a reducer panics between `take()` and re-assigning `Some`, the
actor is permanently poisoned and every subsequent message panics.

**Approach.** Use `std::mem::replace(&mut self.state, default)` if `S: Default`,
or `Option::replace`, or restructure so the reducer takes `&mut S` (conflicts
with the current pure-function framing — revisit only as part of the reducer
signature decision above).

**Acceptance.**
- `Actor.state` is `S`, not `Option<S>`.
- Or, if `Option<S>` remains for a defensible reason, document it explicitly.

---

### Reconsider passing `Action` by value in the `Reducer` trait

**Context.** The trait passes `A` by value with a rationale (rustle_state/src/reducer.rs:18)
that this is fine because actions are small and `Copy`-like. That stops being
true once `Action::InsertString(String)` and `Action::LoadFile(PathBuf)`
land — see Buffer and File-loading tasks above.

**Action.** Revisit the signature. Likely `(S, &A) -> S` is the right answer
for sub-component reducers; root reducer can keep `A` by value if it needs
to consume the action. Decide once, document the choice, remove the now-stale
docstring rationale.

---

### Consider the layout cost of rebuilding the taffy tree every redraw

**Problem.** `Viewport::redraw` calls `self.taffy.clear()` and rebuilds the
entire tree on every redraw (render.rs:297). This is wasted work for an
editor where the tree topology rarely changes — typically only the buffer
text and cursor position change between frames.

**Approach.** Either:
- Cache `NodeId`s on components and update only the styles/text that changed,
  or
- Profile first — for the current tree (root → window → buffer, status_line,
  command_line) this is genuinely negligible and not worth complicating
  things.

**Recommended:** profile first; do nothing if it doesn't show up. Listed for
visibility, not as a definite task.

---

### Consistent error variants

`Error::Input(String)` (rustle_core/src/error.rs:13) takes a raw string
while every other variant uses `#[from]` over a typed inner error.
Inconsistent. Either define an `InputError` type and `#[from]` it, or
accept the inconsistency and document why — but don't widen the pattern
to other variants.

---

### Best-effort cleanup on panic / drop paths

`tui/main.rs:75` and `tui/backend.rs:36–40` use `expect(...)` for terminal
restoration. If those fail during an unrelated panic, the terminal stays
broken *and* the panic message is obscured by a second panic from `expect`.
Replace with `let _ = ...` on the cleanup-only paths — at that point the
terminal is already in an unknown state and there's nothing useful to
recover from a failed restore.

---

### Replace the `backtrace` crate with `std::backtrace::Backtrace`

`std::backtrace::Backtrace` has been stable since Rust 1.65. The
`backtrace` dependency in `rustle_tui` can be dropped. Cosmetic.

---

## Out of scope (for now)

The following are deliberately not on the roadmap. Listed so future-you
doesn't add them mid-flow without thinking about cost, and so README
expectations stay honest.

- **Visual mode / selections.** Needs a selection state model plus
  visual-line vs visual-block semantics. Natural fit after multi-buffer
  lands.
- **Search and replace** (`/`, `?`, `:s/.../.../g`). Pulls in a regex
  dependency and incremental-match-highlighting work.
- **Yank / paste with registers** (`y`, `p`, `"ay`). Needs a register
  store in state, OS clipboard integration, and `"_` / `"+` semantics.
- **Marks** (`m`, `'`, `` ` ``). Persistent positions that survive edits
  — interacts with how cursor positions are represented.
- **Macros** (`q`, `@`). Mostly free once "Keep `Action` serialisable"
  (Architecture) holds and undo/redo lands, but explicitly out of scope
  until both are in place.
- **Line numbers.** Trivial as a render component; interacts with
  scrolling, soft-wrap, and folding when those exist.
- **Syntax highlighting.** tree-sitter is the standard answer; large
  dependency surface; interacts with theming.
- **LSP support.** Large undertaking. Wants the effects layer mature
  first.
- **Themes / colorscheme.** `Color` is currently hard-coded in component
  render fns. Themes mean threading a theme handle through render or
  reading one from state.
- **Plugin system** (Lua, Wasm, …). Not on the table.
- **Folding, soft-wrap, mouse support.** Not on the table short-term.
- **Scrolling.** Out of scope short-term but unavoidable as soon as
  buffers exceed the viewport. Likely the first item to graduate from
  this list.
