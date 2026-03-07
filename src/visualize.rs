use crate::enums::CellType;
use crate::models::{Bomb, GameState, Player, Position};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

const TILE_AIR: &str = "  ";
const TILE_WALL: &str = "🧱";
const TILE_BOX: &str = "📦";
const TILE_BOMB: &str = "💣";
const TILE_EXPLOSION: &str = "💥";
const TILE_ME: &str = "🤖";

const OPPONENT_ICONS: &[&str] = &["👾", "🏃", "🚶", "💃", "🕺", "🦊", "🐼", "🐸"];

pub fn render_field(state: Option<&GameState>) -> String {
    let Some(state) = state else {
        return "<nil game state>\n".to_string();
    };

    let w = state.field.width;
    let h = state.field.height;
    if w <= 0 || h <= 0 {
        return "<empty field>\n".to_string();
    }

    let mut grid = vec![vec![TILE_AIR; w as usize]; h as usize];

    for y in 0..h {
        for x in 0..w {
            let cell = state.field.cell_at(&Position { x, y });
            grid[y as usize][x as usize] = match cell {
                CellType::Wall => TILE_WALL,
                CellType::Box => TILE_BOX,
                _ => TILE_AIR,
            };
        }
    }

    for e in &state.explosions {
        if in_bounds(e, w, h) {
            grid[e.y as usize][e.x as usize] = TILE_EXPLOSION;
        }
    }

    for b in &state.bombs {
        if in_bounds(&b.pos, w, h) {
            grid[b.pos.y as usize][b.pos.x as usize] = TILE_BOMB;
        }
    }

    let icons = opponent_icon_map(state);
    for p in &state.opponents {
        if in_bounds(&p.pos, w, h) {
            if let Some(&icon) = icons.get(&p.id) {
                grid[p.pos.y as usize][p.pos.x as usize] = icon;
            }
        }
    }

    if let Some(me) = &state.me {
        if in_bounds(&me.pos, w, h) {
            grid[me.pos.y as usize][me.pos.x as usize] = TILE_ME;
        }
    }

    let mut sb = String::new();
    let _ = writeln!(sb, "╔{}╗", "══".repeat(w as usize));

    for y in 0..h as usize {
        let _ = writeln!(sb, "║{}║", grid[y].join(""));
    }

    let _ = writeln!(sb, "╚{}╝", "══".repeat(w as usize));

    append_players_section(&mut sb, state, &icons);
    append_bombs_section(&mut sb, &state.bombs);

    let _ = writeln!(
        sb,
        "Legend: [space] AIR  🧱 WALL  📦 BOX  💣 BOMB  💥 EXPLOSION  🤖 ME"
    );
    sb
}

fn append_players_section(
    sb: &mut String,
    state: &GameState,
    icons: &HashMap<String, &'static str>,
) {
    let players = stable_players(state);
    if players.is_empty() {
        return;
    }

    let _ = writeln!(sb, "--- PLAYERS ---");
    for p in players {
        let mut icon = icons.get(&p.id).copied().unwrap_or(OPPONENT_ICONS[0]);
        if let Some(me) = &state.me {
            if p.id == me.id {
                icon = TILE_ME;
            }
        }
        let _ = writeln!(
            sb,
            "{} Player {} | Health: {}, Score: {} | Pos: ({},{})",
            icon,
            short_player_id(&p.id),
            p.health,
            p.score,
            p.pos.x,
            p.pos.y
        );
    }
}

fn append_bombs_section(sb: &mut String, bombs: &[Bomb]) {
    if bombs.is_empty() {
        return;
    }

    let mut sorted = bombs.to_vec();
    sorted.sort_by(|a, b| {
        if a.pos.y != b.pos.y {
            a.pos.y.cmp(&b.pos.y)
        } else if a.pos.x != b.pos.x {
            a.pos.x.cmp(&b.pos.x)
        } else {
            a.fuse.cmp(&b.fuse)
        }
    });

    let _ = writeln!(sb, "--- BOMBS ---");
    for b in sorted {
        let _ = writeln!(sb, "💣 at ({},{}) | Fuse: {}", b.pos.x, b.pos.y, b.fuse);
    }
}

fn stable_players(state: &GameState) -> Vec<Player> {
    if !state.players.is_empty() {
        let mut players = state.players.clone();
        players.sort_by(|a, b| a.id.cmp(&b.id));
        return players;
    }

    let mut players = Vec::with_capacity(state.opponents.len() + 1);
    if let Some(me) = &state.me {
        players.push(me.clone());
    }
    players.extend(state.opponents.iter().cloned());
    players.sort_by(|a, b| a.id.cmp(&b.id));
    players
}

fn opponent_icon_map(state: &GameState) -> HashMap<String, &'static str> {
    let mut icons = HashMap::new();
    let mut taken = HashSet::new();

    if let Some(me) = &state.me {
        taken.insert(me.id.clone());
    }

    let mut opponent_ids = Vec::with_capacity(state.opponents.len());
    for p in &state.opponents {
        if taken.contains(&p.id) {
            continue;
        }
        taken.insert(p.id.clone());
        opponent_ids.push(p.id.clone());
    }
    opponent_ids.sort();

    for (i, id) in opponent_ids.iter().enumerate() {
        icons.insert(id.clone(), OPPONENT_ICONS[i % OPPONENT_ICONS.len()]);
    }

    for p in stable_players(state) {
        if icons.contains_key(&p.id) {
            continue;
        }
        let len = icons.len();
        icons.insert(p.id.clone(), OPPONENT_ICONS[len % OPPONENT_ICONS.len()]);
    }

    icons
}

fn short_player_id(id: &str) -> String {
    if id.is_empty() {
        return "<unknown>".to_string();
    }
    const N: usize = 4;
    let chars_count = id.chars().count();
    if chars_count <= N {
        id.to_string()
    } else {
        let start_idx = chars_count - N;
        let suffix: String = id.chars().skip(start_idx).collect();
        format!("...{}", suffix)
    }
}

pub fn print_field(state: Option<&GameState>) {
    print!("{}", render_field(state));
}

fn in_bounds(pos: &Position, width: i32, height: i32) -> bool {
    pos.x >= 0 && pos.x < width && pos.y >= 0 && pos.y < height
}
