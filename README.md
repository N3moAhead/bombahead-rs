# bombahead-rs

Rust SDK for building Bomberman bots that connect to a Bombahead game server over WebSockets.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bombahead-rs = { path = "../path/to/bombahead-rs" } # Or whatever the registry path becomes
```

## Quick Start

1. Implement the `bombahead_rs::Bot` trait.
2. Call `bombahead_rs::run(your_bot)` from `main`.
3. Run your program.

## Core API

### Run

```rust
pub fn run<B: Bot>(mut user_bot: B)
```

Starts the bot client loop:
- Connects to the server over WebSocket (TLS is supported).
- Marks the player as ready.
- Receives game state messages.
- Builds `GameHelpers`.
- Calls `user_bot.get_next_move(&state, &helpers)`.
- Sends the returned action to the server.
- Re-readies automatically after a `back_to_lobby` event.

This function blocks the current thread until the connection closes or a fatal runtime error occurs.

### Bot Trait

```rust
pub trait Bot {
    fn get_next_move(&mut self, state: &GameState, helpers: &GameHelpers) -> Action;
}
```

Implement this trait to provide your bot logic.

### GameHelpers

```rust
pub struct GameHelpers<'a> {
    pub state: &'a GameState,
}
```

Utility bound to the current game state for pathing and safety checks. Created automatically by the `run` loop before delegating to `get_next_move`.

### Field Visualization

The SDK provides utilities to visualize the game board in the console.

```rust
pub fn render_field(state: Option<&GameState>) -> String
pub fn print_field(state: Option<&GameState>)
```

It outputs a styled ASCII grid with the legend. Example usage:

```rust
fn get_next_move(&mut self, state: &GameState, _helpers: &GameHelpers) -> Action {
    bombahead_rs::print_field(Some(state));
    Action::DoNothing
}
```

## Types and Models

### Action

```rust
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    PlaceBomb,
    DoNothing,
}
```

### CellType

```rust
pub enum CellType {
    Air,
    Wall,
    Box,
}
```

### Core Structs

```rust
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Player {
    pub id: String,
    pub pos: Position,
    pub health: i32,
    pub score: i32,
}

pub struct Bomb {
    pub pos: Position,
    pub fuse: i32,
}

pub struct Field {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<CellType>,
}

pub struct GameState {
    pub current_tick: i32,
    pub me: Option<Player>,
    pub opponents: Vec<Player>,
    pub players: Vec<Player>,
    pub field: Field,
    pub bombs: Vec<Bomb>,
    pub explosions: Vec<Position>,
}
```

## Helper Methods

The `GameHelpers` struct provides these core methods:

- `is_walkable(&self, pos: &Position) -> bool`: Checks if a square is Air and unblocked by bombs.
- `get_adjacent_walkable_positions(&self, pos: &Position) -> Vec<Position>`: Yields valid orthagonal steps.
- `get_next_action_towards(&self, start: &Position, target: &Position) -> Action`: A BFS pathfinder that yields the next optimal step.
- `is_safe(&self, pos: &Position) -> bool`: Assesses if a tile is clear of immediate explosion lanes.
- `get_nearest_safe_position(&self, start: &Position) -> Position`: Locates the nearest survivable tile.
- `find_nearest_box(&self, start: &Position) -> Option<Position>`: Scans for the closest destroyable Box.

## Minimal Bot Example

This example demonstrates how to evade danger and hunt boxes:

```rust
use bombahead_rs::{Action, Bot, GameState, GameHelpers};

struct SimpleBot;

impl Bot for SimpleBot {
    fn get_next_move(&mut self, state: &GameState, h: &GameHelpers) -> Action {
        let me = match &state.me {
            Some(player) => &player.pos,
            None => return Action::DoNothing,
        };

        // Run from explosions
        if !h.is_safe(me) {
            let safe = h.get_nearest_safe_position(me);
            return h.get_next_action_towards(me, &safe);
        }

        // Hunt boxes
        if let Some(box_pos) = h.find_nearest_box(me) {
            let dist = me.distance_to(&box_pos);
            if dist == 1 {
                return Action::PlaceBomb;
            } else if dist > 1 {
                return h.get_next_action_towards(me, &box_pos);
            }
        }

        Action::DoNothing
    }
}

fn main() {
    println!("Starting SimpleBot...");
    bombahead_rs::run(SimpleBot);
}
```
