use crate::enums::CellType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn distance_to(&self, other: &Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub pos: Position,
    pub health: i32,
    pub score: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bomb {
    pub pos: Position,
    pub fuse: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub width: i32,
    pub height: i32,
    pub cells: Vec<CellType>,
}

impl Field {
    pub fn cell_at(&self, pos: &Position) -> CellType {
        if pos.x < 0 || pos.x >= self.width || pos.y < 0 || pos.y >= self.height {
            return CellType::Wall;
        }
        let idx = (pos.y * self.width + pos.x) as usize;
        self.cells.get(idx).copied().unwrap_or(CellType::Wall)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub current_tick: i32,
    pub me: Option<Player>,
    pub opponents: Vec<Player>,
    pub players: Vec<Player>,
    pub field: Field,
    pub bombs: Vec<Bomb>,
    pub explosions: Vec<Position>,
}
