pub mod board;
pub mod piece;
pub mod square;
pub mod move_generation;
use crate::board::Board;
use crate::square::Square;
use crate::move_generation::{Move, MoveGenerator};


fn main() {
    let mut board = Board::starting_position();
    board.move_piece(Move::from_square(Square::E2, Square::E4));
    board.move_piece(Move::from_square(Square::E7, Square::E5));
    board.move_piece(Move::from_square(Square::G1, Square::F3));
    board.move_piece(Move::from_square(Square::B8, Square::C6));
    println!("Talia Chess Engine: v0.0.1");
    println!("{board}");

    let mut generator = MoveGenerator::new(board);
    generator.generate_moves();

    for mv in generator.moves {
        println!("{:?}", mv);
    }
}
