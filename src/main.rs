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
pub mod piece_square_table;
use crate::game_manager::Game;
use crate::piece::Color;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Talia Chess Engine: v1.0.1");

    let search_depth = 6;
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut game = Game::try_from_fen(Some(fen), Some(Color::White), search_depth)?;
    game.start_game()?;

    Ok(())
}
