use std::error::Error;

pub mod board;
pub mod board_builder;
pub mod errors;
pub mod evaluate;
pub mod game_manager;
pub mod move_generation;
pub mod piece;
pub mod search;
pub mod square;
use crate::game_manager::start_new_game;
use crate::piece::Color;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Talia Chess Engine: v1.0.0");

    let search_depth = 5;
    start_new_game(None, Color::White, search_depth)?;

    Ok(())
}
