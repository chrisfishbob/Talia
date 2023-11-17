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
use crate::piece::Color;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Talia Chess Engine: v1.0.1");

    let search_depth = 6;
    let fen = "r3k2r/p1ppqpbp/bn2pnp1/3PN3/1p2P3/2N2Q2/PPPBBPPP/R3K2R w KQkq - 0 1";
    game_manager::start_new_game(Some(fen), Some(Color::White), search_depth)?;

    Ok(())
}
