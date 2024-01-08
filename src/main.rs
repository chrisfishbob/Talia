use anyhow::Result;

pub mod board;
pub mod board_builder;
pub mod bot;
pub mod evaluate;
pub mod game_manager;
pub mod move_generation;
pub mod piece;
pub mod piece_square_table;
pub mod search;
pub mod square;
use crate::bot::Bot;
// use crate::game_manager::Game;
// use crate::piece::Color;

fn main() -> Result<()> {
    println!("Talia Chess Engine: v1.1.1");

    // let search_depth = 6;
    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // let mut game = Game::try_from_fen(fen, Some(Color::White), search_depth)?;
    // game.start_game()?;
    let mut bot = Bot::new();
    bot.start_uci()?;

    Ok(())
}
