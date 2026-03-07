use crate::enums::{Action, CellType};
use crate::models::{GameState, Position};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct GameHelpers<'a> {
    pub state: &'a GameState,
}

const DEFAULT_BOMB_RANGE: i32 = 2;

impl<'a> GameHelpers<'a> {
    pub fn new(state: &'a GameState) -> Self {
        Self { state }
    }

    pub fn is_walkable(&self, pos: &Position) -> bool {
        if pos.x < 0
            || pos.x >= self.state.field.width
            || pos.y < 0
            || pos.y >= self.state.field.height
        {
            return false;
        }

        let cell = self.state.field.cell_at(pos);
        if cell == CellType::Wall || cell == CellType::Box {
            return false;
        }

        for bomb in &self.state.bombs {
            if &bomb.pos == pos {
                return false;
            }
        }

        true
    }

    pub fn get_adjacent_walkable_positions(&self, pos: &Position) -> Vec<Position> {
        let candidates = [
            Position {
                x: pos.x,
                y: pos.y - 1,
            },
            Position {
                x: pos.x + 1,
                y: pos.y,
            },
            Position {
                x: pos.x,
                y: pos.y + 1,
            },
            Position {
                x: pos.x - 1,
                y: pos.y,
            },
        ];

        candidates
            .into_iter()
            .filter(|p| self.is_walkable(p))
            .collect()
    }

    pub fn get_next_action_towards(&self, start: &Position, target: &Position) -> Action {
        if start == target {
            return Action::DoNothing;
        }

        let prev = self.bfs(start, |pos| pos == target, false);
        if prev.is_empty() {
            return Action::DoNothing;
        }

        let path = Self::rebuild_path(start, target, &prev);
        if path.len() < 2 {
            return Action::DoNothing;
        }

        Self::action_from_step(&path[0], &path[1])
    }

    pub fn is_safe(&self, pos: &Position) -> bool {
        if pos.x < 0
            || pos.x >= self.state.field.width
            || pos.y < 0
            || pos.y >= self.state.field.height
        {
            return false;
        }

        for b in &self.state.bombs {
            if &b.pos == pos {
                return false;
            }
        }

        let danger = self.compute_danger_positions();
        if danger.contains(pos) {
            return false;
        }

        true
    }

    pub fn get_nearest_safe_position(&self, start: &Position) -> Position {
        if self.is_walkable(start) && self.is_safe(start) {
            return *start;
        }

        let prev = self.bfs(
            start,
            |pos| self.is_walkable(pos) && self.is_safe(pos),
            true,
        );
        if prev.is_empty() {
            return *start;
        }

        let mut queue = VecDeque::new();
        queue.push_back(*start);
        let mut visited = HashSet::new();
        visited.insert(*start);

        while let Some(cur) = queue.pop_front() {
            if self.is_walkable(&cur) && self.is_safe(&cur) {
                return cur;
            }

            for next in self.get_adjacent_walkable_positions(&cur) {
                if !visited.contains(&next) {
                    visited.insert(next);
                    queue.push_back(next);
                }
            }
        }

        *start
    }

    pub fn find_nearest_box(&self, start: &Position) -> Option<Position> {
        let mut queue = VecDeque::new();
        queue.push_back(*start);
        let mut visited = HashSet::new();
        visited.insert(*start);

        while let Some(cur) = queue.pop_front() {
            if self.state.field.cell_at(&cur) == CellType::Box {
                return Some(cur);
            }

            let neighbors = [
                Position {
                    x: cur.x,
                    y: cur.y - 1,
                },
                Position {
                    x: cur.x + 1,
                    y: cur.y,
                },
                Position {
                    x: cur.x,
                    y: cur.y + 1,
                },
                Position {
                    x: cur.x - 1,
                    y: cur.y,
                },
            ];

            for next in neighbors {
                if next.x < 0
                    || next.x >= self.state.field.width
                    || next.y < 0
                    || next.y >= self.state.field.height
                {
                    continue;
                }
                if visited.contains(&next) {
                    continue;
                }

                let cell = self.state.field.cell_at(&next);
                if cell == CellType::Wall {
                    continue;
                }

                visited.insert(next);
                queue.push_back(next);
            }
        }

        None
    }

    fn bfs<F>(
        &self,
        start: &Position,
        goal: F,
        allow_unsafe_start: bool,
    ) -> HashMap<Position, Position>
    where
        F: Fn(&Position) -> bool,
    {
        let mut prev = HashMap::new();
        if !allow_unsafe_start && !self.is_walkable(start) {
            return prev;
        }

        let mut queue = VecDeque::new();
        queue.push_back(*start);
        let mut visited = HashSet::new();
        visited.insert(*start);

        while let Some(cur) = queue.pop_front() {
            if cur != *start && goal(&cur) {
                return prev;
            }

            for next in self.get_adjacent_walkable_positions(&cur) {
                if visited.contains(&next) {
                    continue;
                }
                visited.insert(next);
                prev.insert(next, cur);
                queue.push_back(next);
            }
        }

        HashMap::new()
    }

    fn rebuild_path(
        start: &Position,
        target: &Position,
        prev: &HashMap<Position, Position>,
    ) -> Vec<Position> {
        let mut path = vec![*target];
        let mut cur = *target;
        while cur != *start {
            if let Some(&p) = prev.get(&cur) {
                cur = p;
                path.push(cur);
            } else {
                return vec![];
            }
        }
        path.reverse();
        path
    }

    fn action_from_step(from: &Position, to: &Position) -> Action {
        if to.x == from.x && to.y == from.y - 1 {
            Action::MoveUp
        } else if to.x == from.x + 1 && to.y == from.y {
            Action::MoveRight
        } else if to.x == from.x && to.y == from.y + 1 {
            Action::MoveDown
        } else if to.x == from.x - 1 && to.y == from.y {
            Action::MoveLeft
        } else {
            Action::DoNothing
        }
    }

    fn compute_danger_positions(&self) -> HashSet<Position> {
        let mut danger = HashSet::new();
        let mut bomb_index = HashMap::new();
        for (i, b) in self.state.bombs.iter().enumerate() {
            bomb_index.insert(b.pos, i);
        }

        let mut triggered = vec![false; self.state.bombs.len()];
        let mut queue = VecDeque::new();

        for e in &self.state.explosions {
            danger.insert(*e);
            if let Some(&idx) = bomb_index.get(e) {
                if !triggered[idx] {
                    triggered[idx] = true;
                    queue.push_back(idx);
                }
            }
        }

        for (i, b) in self.state.bombs.iter().enumerate() {
            if b.fuse <= 1 && !triggered[i] {
                triggered[i] = true;
                queue.push_back(i);
            }
        }

        while let Some(idx) = queue.pop_front() {
            let blast = self.blast_cells(&self.state.bombs[idx].pos);
            for cell in blast {
                danger.insert(cell);
                if let Some(&hit_idx) = bomb_index.get(&cell) {
                    if !triggered[hit_idx] {
                        triggered[hit_idx] = true;
                        queue.push_back(hit_idx);
                    }
                }
            }
        }

        danger
    }

    fn blast_cells(&self, origin: &Position) -> Vec<Position> {
        let mut cells = vec![*origin];
        let directions = [
            Position { x: 0, y: -1 },
            Position { x: 1, y: 0 },
            Position { x: 0, y: 1 },
            Position { x: -1, y: 0 },
        ];

        for d in directions {
            for step in 1..=DEFAULT_BOMB_RANGE {
                let pos = Position {
                    x: origin.x + d.x * step,
                    y: origin.y + d.y * step,
                };

                if pos.x < 0
                    || pos.x >= self.state.field.width
                    || pos.y < 0
                    || pos.y >= self.state.field.height
                {
                    break;
                }

                let cell = self.state.field.cell_at(&pos);
                if cell == CellType::Wall {
                    break;
                }

                cells.push(pos);
                if cell == CellType::Box {
                    break;
                }
            }
        }

        cells
    }
}
