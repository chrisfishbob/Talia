use anyhow::Result;
use clap::Parser;

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
use crate::game_manager::Game;
use crate::piece::Color;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value_t = false)]
    cli: bool,
}

fn main() -> Result<()> {
    println!("Talia Chess Engine: v1.1.1");
    let args = Args::parse();

    if args.cli {
        let search_depth = 6;
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut game = Game::try_from_fen(fen, Some(Color::White), search_depth)?;
        game.start_game()?;
    } else {
        let mut bot = Bot::new();
        bot.start_uci()?;
    }

    Ok(())
}
