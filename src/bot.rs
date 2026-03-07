use crate::enums::Action;
use crate::helpers::GameHelpers;
use crate::models::GameState;

pub trait Bot {
    fn get_next_move(&mut self, state: &GameState, helpers: &GameHelpers) -> Action;
}
