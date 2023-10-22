use std::error::Error;

pub mod board;
pub mod board_builder;
pub mod errors;
pub mod move_generation;
pub mod piece;
pub mod square;
use crate::board::Board;
use crate::board_builder::BoardBuilder;
use crate::move_generation::{Flag, Move, MoveGenerator};
use crate::square::Square;

fn main() -> Result<(), Box<dyn Error>> {
    let board: Board = BoardBuilder::from_starting_position()
        .make_move(Move::from_square(Square::E2, Square::E4, Flag::None))
        .make_move(Move::from_square(Square::E7, Square::E5, Flag::None))
        .make_move(Move::from_square(Square::G1, Square::F3, Flag::None))
        .make_move(Move::from_square(Square::B8, Square::C6, Flag::None))
        .make_move(Move::from_square(Square::F1, Square::C4, Flag::None))
        .make_move(Move::from_square(Square::F8, Square::C5, Flag::None))
        .make_move(Move::from_square(Square::E1, Square::G1, Flag::KingsideCastle))
        .try_into()?;

    println!("Talia Chess Engine: v0.0.1");
    println!("{board}");

    let mut generator = MoveGenerator::new(board.clone());
    generator.generate_moves();

    generator.moves.iter().for_each(|mv| {
        println!("{:?}", mv);
    });

    Ok(())
}
